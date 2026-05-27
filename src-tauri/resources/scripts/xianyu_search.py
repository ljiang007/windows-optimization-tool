"""
咸鱼(goofish.com)商品搜索价格爬取脚本
使用方式: python xianyu_search.py <关键词>
输出格式: {"xianyu": 最低价格数字} 或 {"error": "错误信息"}

首次运行会弹出浏览器窗口，登录咸鱼后自动保存会话，后续静默运行。

依赖安装:
  pip install playwright
  playwright install chromium
"""

import sys
import json
import os
import re
import time
import argparse
from urllib.parse import unquote, urlparse, parse_qs
from pathlib import Path

UA = (
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) '
    'AppleWebKit/537.36 (KHTML, like Gecko) '
    'Chrome/125.0.0.0 Safari/537.36'
)
STEALTH = "Object.defineProperty(navigator,'webdriver',{get:()=>undefined})"
SESSION_FILE = Path(os.environ.get('APPDATA', '.')) / 'system_toolbox' / 'xianyu_session.json'
PROFILE_FILE = Path(os.environ.get('APPDATA', '.')) / 'system_toolbox' / 'xianyu_profile.json'
FILTER_CONFIG_FILE = Path(__file__).with_name('xianyu_filter_keywords.json')
LOGIN_COOKIE_NAMES = (
    'tracknick', '_nk_', 'lgc', 'sn', 'unb', 'dnk', 'lid', 'cookie17',
    '_l_g_', 'sgcookie', 'skt', 'csg',
)
FILTER_CONFIG_CACHE = None
LOGIN_PROMPTS = ('立即登录', '扫码登录', '手机扫码登录', '登录/注册', '请登录')
LOGIN_SCAN_TEXTS = ('扫码成功', '扫描成功', '已扫码', '请在手机上确认', '手机确认', '确认登录')
LOGIN_SUCCESS_TEXTS = ('登录成功', '授权成功', '已确认', '确认成功', '登录完成')
DETAIL_API_MARKER = 'mtop.taobao.idle.pc.detail'
MAX_DETAIL_CANDIDATES = 10


def _emit(payload: dict):
    print(json.dumps(payload, ensure_ascii=True), flush=True)


def _emit_log(message: str, level: str = 'info', **extra):
    payload = {'status': 'log', 'type': level, 'message': message}
    payload.update(extra)
    _emit(payload)


def _parse_links_from_response(raw_json: str) -> list:
    links = []
    for raw in re.findall(r'https?:\\?/\\?/[^"\\]+', raw_json):
        url = _normalize_item_url(raw)
        if url:
            links.append(url)

    for item_id in re.findall(r'"(?:itemId|item_id)"\s*:\s*"?(\d{6,})"?', raw_json):
        links.append(f'https://www.goofish.com/item?id={item_id}')

    return _unique(links)


def _extract_item_id(value: str) -> str:
    if not value:
        return ''
    url = str(value).replace('\\/', '/').strip()
    try:
        query = parse_qs(urlparse(url).query)
        for key in ('id', 'itemId', 'item_id'):
            values = query.get(key)
            if values and str(values[0]).strip():
                return str(values[0]).strip()
    except Exception:
        pass

    match = re.search(r'(?:itemId|item_id|id)[=/](\d{6,})', url)
    return match.group(1) if match else ''


def _normalize_item_url(value: str) -> str:
    if not value:
        return ''
    url = str(value).replace('\\/', '/').strip()
    if url.startswith('//'):
        url = f'https:{url}'
    item_id = _extract_item_id(url)
    if url.startswith('fleamarket://') and item_id:
        return f'https://www.goofish.com/item?id={item_id}'
    if 'goofish.com' not in url or not item_id:
        return ''

    category_id = ''
    try:
        query = parse_qs(urlparse(url).query)
        for key in ('categoryId', 'category_id', 'channelCatId'):
            values = query.get(key)
            if values and str(values[0]).strip():
                category_id = str(values[0]).strip()
                break
    except Exception:
        pass

    normalized = f'https://www.goofish.com/item?id={item_id}'
    if category_id:
        normalized = f'{normalized}&categoryId={category_id}'
    return normalized


def _loads_json_like(raw_json):
    if isinstance(raw_json, dict):
        return raw_json
    text = str(raw_json or '').strip()
    if not text:
        return None
    try:
        return json.loads(text)
    except Exception:
        pass

    match = re.match(r'^[\w$]+\((.*)\)\s*;?\s*$', text, re.S)
    if not match:
        return None
    try:
        return json.loads(match.group(1))
    except Exception:
        return None


def _parse_price(value):
    if value is None:
        return None
    if isinstance(value, (int, float)):
        price = float(value)
    else:
        text = str(value).replace(',', '').strip()
        match = re.search(r'(\d+(?:\.\d+)?)', text)
        if not match:
            return None
        price = float(match.group(1))
        if '万' in text:
            price *= 10000
    if 1 < price < 999999:
        return price
    return None


def _format_price(price) -> str:
    if price is None:
        return '--'
    if float(price).is_integer():
        return str(int(price))
    return f'{price:.2f}'.rstrip('0').rstrip('.')


def _unique(values: list) -> list:
    seen = set()
    results = []
    for value in values:
        if not value or value in seen:
            continue
        seen.add(value)
        results.append(value)
    return results


def _unique_items(items: list) -> list:
    seen = set()
    results = []
    for item in items:
        url = item.get('url')
        price = _parse_price(item.get('price'))
        if not url or price is None or url in seen:
            continue
        seen.add(url)
        results.append({'url': url, 'price': price})
    return results


def _sort_items_by_price(items: list) -> list:
    return sorted(_unique_items(items), key=lambda item: item['price'])


def _normalize_match_text(value: str) -> str:
    return re.sub(r'[\s\W_]+', '', str(value or ''), flags=re.UNICODE).upper()


def _text_matches_keyword(text: str, keyword: str) -> bool:
    normalized_text = _normalize_match_text(text)
    normalized_keyword = _normalize_match_text(keyword)
    if not normalized_keyword:
        return True
    if normalized_keyword in normalized_text:
        return True

    tokens = [
        _normalize_match_text(token)
        for token in re.split(r'\s+', str(keyword or '').strip())
        if _normalize_match_text(token)
    ]
    return bool(tokens) and all(token in normalized_text for token in tokens)


def _find_rejected_term(text: str, terms: list) -> str:
    raw_text = str(text or '')
    normalized_text = _normalize_match_text(raw_text)
    for term in terms or []:
        raw_term = str(term or '').strip()
        if not raw_term:
            continue
        if raw_term in raw_text or _normalize_match_text(raw_term) in normalized_text:
            return raw_term
    return ''


def _parse_share_info(item: dict) -> dict:
    share_data = item.get('shareData') or {}
    share_info = share_data.get('shareInfoJsonString') if isinstance(share_data, dict) else None
    parsed = _loads_json_like(share_info)
    return parsed if isinstance(parsed, dict) else {}


def _collect_rich_text(raw_value) -> str:
    parsed = _loads_json_like(raw_value)
    if not isinstance(parsed, dict):
        return ''

    texts = []

    def walk(node):
        if isinstance(node, dict):
            text = node.get('text')
            if isinstance(text, str) and text.strip():
                texts.append(text)
            for child in node.get('children') or []:
                walk(child)
        elif isinstance(node, list):
            for child in node:
                walk(child)

    walk(parsed)
    return '\n'.join(texts)


def _parse_detail_response(raw_json: str, item_url: str = '') -> dict:
    payload = _loads_json_like(raw_json)
    if not isinstance(payload, dict):
        return None

    data = payload.get('data') or {}
    if not isinstance(data, dict):
        return None
    item = data.get('itemDO') or {}
    if not isinstance(item, dict) or not item:
        return None

    share_info = _parse_share_info(item)
    content_params = share_info.get('contentParams') or {}
    if not isinstance(content_params, dict):
        content_params = {}
    main_params = share_info.get('mainParams') or content_params.get('mainParams') or {}
    if not isinstance(main_params, dict):
        main_params = {}
    share_extra = main_params.get('extra') or {}
    if not isinstance(share_extra, dict):
        share_extra = {}
    price_candidates = [
        ('data.itemDO.soldPrice', item.get('soldPrice')),
        ('shareData.contentParams.mainParams.extra.soldPrice', share_extra.get('soldPrice')),
        ('data.itemDO.price', item.get('price')),
        ('data.itemDO.currentPrice', item.get('currentPrice')),
    ]

    price = None
    price_source = ''
    for source, value in price_candidates:
        price = _parse_price(value)
        if price is not None:
            price_source = source
            break

    if price is None:
        return None

    track_params = data.get('trackParams') or item.get('trackParams') or {}
    if not isinstance(track_params, dict):
        track_params = {}
    item_id = item.get('itemId') or track_params.get('itemId') or _extract_item_id(item_url)
    text_parts = [
        item.get('title'),
        item.get('desc'),
        item.get('itemStatusStr'),
        _collect_rich_text(item.get('richTextDesc')),
        main_params.get('content'),
    ]

    header_params = share_info.get('headerParams') or content_params.get('headerParams') or {}
    if not isinstance(header_params, dict):
        header_params = {}
    text_parts.extend([header_params.get('title'), header_params.get('subTitle')])

    for label in item.get('itemLabelExtList') or []:
        if isinstance(label, dict):
            text_parts.extend([label.get('text'), label.get('valueText'), label.get('propertyText')])
    for label in item.get('cpvLabels') or []:
        if isinstance(label, dict):
            text_parts.extend([label.get('propertyName'), label.get('valueName')])

    detail_text = '\n'.join(str(part) for part in text_parts if part)
    return {
        'url': _normalize_item_url(item_url),
        'price': price,
        'price_source': price_source,
        'item_id': str(item_id) if item_id else '',
        'title': str(item.get('title') or ''),
        'text': detail_text,
    }


def _read_detail_item_from_api(ctx, url: str, keyword: str, rejected_terms: list) -> dict:
    detail_url = _normalize_item_url(url)
    if not detail_url:
        return None

    expected_item_id = _extract_item_id(detail_url)
    api_payloads = []
    page = None
    try:
        page = ctx.new_page()
        page.add_init_script(STEALTH)

        def on_detail_response(resp):
            if resp.status != 200 or DETAIL_API_MARKER not in resp.url:
                return
            try:
                api_payloads.append(resp.text())
            except Exception:
                pass

        page.on('response', on_detail_response)
        try:
            page.goto(detail_url, wait_until='domcontentloaded', timeout=25000)
        except Exception:
            _emit_log(f'详情页打开超时，继续等待详情接口：{detail_url}', 'warning', url=detail_url)

        deadline = time.time() + 12
        while time.time() < deadline and not api_payloads:
            page.wait_for_timeout(500)

        parsed_items = []
        for raw in api_payloads:
            item = _parse_detail_response(raw, detail_url)
            if not item:
                continue
            actual_item_id = item.get('item_id')
            if expected_item_id and actual_item_id and expected_item_id != actual_item_id:
                continue
            parsed_items.append(item)

        if not parsed_items:
            _emit_log(f'未从详情接口拿到价格，跳过：{detail_url}', 'warning', url=detail_url)
            return None

        item = parsed_items[0]
        detail_text = item.get('text') or item.get('title') or ''
        if not _text_matches_keyword(detail_text, keyword):
            _emit_log(f'跳过商品：详情文本不含关键词，链接：{detail_url}', 'warning', url=detail_url)
            return None

        rejected_term = _find_rejected_term(detail_text, rejected_terms)
        if rejected_term:
            _emit_log(f'跳过商品：命中过滤词「{rejected_term}」，链接：{detail_url}', 'warning', url=detail_url)
            return None

        _emit_log(
            f'详情接口价格：￥{_format_price(item.get("price"))}，来源：{item.get("price_source")}，链接：{detail_url}',
            'info',
            url=detail_url,
        )
        return {'url': detail_url, 'price': item.get('price')}
    except Exception as exc:
        _emit_log(f'详情接口读取失败：{detail_url}；{exc}', 'warning', url=detail_url)
        return None
    finally:
        if page:
            try:
                page.close()
            except Exception:
                pass


def _load_filter_config() -> dict:
    global FILTER_CONFIG_CACHE
    if FILTER_CONFIG_CACHE is not None:
        return FILTER_CONFIG_CACHE

    try:
        FILTER_CONFIG_CACHE = json.loads(FILTER_CONFIG_FILE.read_text(encoding='utf-8'))
    except Exception as exc:
        _emit_log(f'过滤词配置读取失败，使用空配置：{exc}', 'warning')
        FILTER_CONFIG_CACHE = {}
    return FILTER_CONFIG_CACHE


def _get_rejected_terms(category: str = '') -> list:
    config = _load_filter_config()
    terms = []

    # 默认不走 default；未传分类时，合并五大核心品类过滤词。
    if category:
        terms.extend(config.get(category) or [])
    else:
        for key in ('cpu', 'gpu', 'memory', 'ssd', 'motherboard'):
            terms.extend(config.get(key) or [])

    return _unique([str(term).strip() for term in terms if str(term).strip()])


def _run_search(p, keyword: str, session_file: Path, category: str = ''):
    """headless 搜索，返回 (价格列表, 是否需要登录, 商品链接列表, 带价格商品列表)"""
    item_links = []
    priced_items = []
    rejected_terms = _get_rejected_terms(category)

    browser = p.chromium.launch(headless=True, args=['--no-sandbox', '--disable-dev-shm-usage'])
    ctx_kwargs = {'user_agent': UA, 'viewport': {'width': 1280, 'height': 800}}
    if session_file.exists():
        ctx_kwargs['storage_state'] = str(session_file)

    ctx = browser.new_context(**ctx_kwargs)

    def on_response(resp):
        if resp.status != 200:
            return
        if 'idlemtopsearch' not in resp.url and 'idlefish' not in resp.url:
            return
        try:
            raw = resp.text()
            item_links.extend(_parse_links_from_response(raw))
        except Exception:
            pass

    page = ctx.new_page()
    page.add_init_script(STEALTH)
    page.on('response', on_response)

    try:
        page.goto(
            f'https://www.goofish.com/search?q={keyword}',
            wait_until='networkidle',
            timeout=30000,
        )
    except Exception:
        pass

    try:
        body = page.inner_text('body')
    except Exception:
        body = ''

    try:
        dom_links = page.eval_on_selector_all(
            'a[href]',
            """
            els => els.map((a) => {
              try { return new URL(a.getAttribute('href'), location.href).href } catch { return '' }
            }).filter((href) => href.includes('goofish.com') && (href.includes('item') || href.includes('detail')))
            """,
        )
        item_links.extend(_normalize_item_url(link) for link in dom_links)
    except Exception:
        pass

    normalized_links = [_normalize_item_url(link) for link in item_links]
    candidate_links = _unique(link for link in normalized_links if link)
    for link in candidate_links[:MAX_DETAIL_CANDIDATES]:
        item = _read_detail_item_from_api(ctx, link, keyword, rejected_terms)
        if item:
            priced_items.append(item)

    browser.close()

    needs_login = not candidate_links and not priced_items and _text_needs_login(body)
    return [], needs_login, candidate_links, _unique_items(priced_items)


def _collect_profile(ctx, page=None) -> dict:
    cookies = ctx.cookies()
    candidates = {}
    for cookie in cookies:
        name = cookie.get('name')
        if name in LOGIN_COOKIE_NAMES:
            candidates[name] = unquote(cookie.get('value') or '')

    storage = {}
    if page:
        try:
            storage = page.evaluate("""
                () => {
                  const pick = (source) => {
                    const result = {};
                    for (let i = 0; i < source.length; i += 1) {
                      const key = source.key(i);
                      if (!key || !/user|nick|member|login|goofish|xianyu|idle/i.test(key)) continue;
                      result[key] = source.getItem(key);
                    }
                    return result;
                  };
                  return {
                    localStorage: pick(window.localStorage),
                    sessionStorage: pick(window.sessionStorage),
                  };
                }
            """)
        except Exception:
            storage = {}

    return {
        'display_name': '已登录',
        'candidate_cookies': candidates,
        'cookie_names': sorted({cookie.get('name') for cookie in cookies if cookie.get('name')}),
        'storage': storage,
    }


def _has_login_cookie(profile: dict) -> bool:
    candidates = profile.get('candidate_cookies') or {}
    return any(str(value).strip() for value in candidates.values())


def _text_needs_login(text: str) -> bool:
    return any(prompt in (text or '') for prompt in LOGIN_PROMPTS)


def _has_any_text(text: str, candidates: tuple) -> bool:
    return any(candidate in (text or '') for candidate in candidates)


def _is_goofish_page(url: str) -> bool:
    try:
        return urlparse(url).hostname in ('www.goofish.com', 'goofish.com')
    except Exception:
        return False


def _read_login_page_signals(page) -> tuple:
    text_parts = []
    frame_urls = []
    for frame in page.frames:
        try:
            frame_urls.append(frame.url)
        except Exception:
            pass
        try:
            text_parts.append(frame.inner_text('body', timeout=1000))
        except Exception:
            pass

    body_text = '\n'.join(text for text in text_parts if text)
    scan_seen = _has_any_text(body_text, LOGIN_SCAN_TEXTS)
    success_seen = _has_any_text(body_text, LOGIN_SUCCESS_TEXTS) or any(_is_goofish_page(url) for url in frame_urls)
    return body_text, scan_seen, success_seen, frame_urls


def _verify_login_context(ctx, allow_text_fallback: bool = False) -> tuple:
    """校验扫码后的同一浏览器上下文是否已经拿到登录态。"""
    profile = _collect_profile(ctx)
    if _has_login_cookie(profile):
        return True, profile, '检测到登录 Cookie'

    verify_page = None
    try:
        verify_page = ctx.new_page()
        verify_page.add_init_script(STEALTH)
        verify_page.goto('https://www.goofish.com/', wait_until='domcontentloaded', timeout=15000)
        verify_page.wait_for_timeout(1500)
        verify_text = verify_page.inner_text('body')
        profile = _collect_profile(ctx, verify_page)
        if _has_login_cookie(profile):
            return True, profile, '首页校验检测到登录 Cookie'
        if allow_text_fallback and verify_text and not _text_needs_login(verify_text):
            return True, profile, '首页未出现登录提示'
    except Exception as exc:
        return False, profile, f'登录态校验失败：{exc}'
    finally:
        if verify_page:
            try:
                verify_page.close()
            except Exception:
                pass

    return False, profile, '仍未检测到有效登录态'


def _format_cookie_names(profile: dict) -> str:
    names = profile.get('cookie_names') or []
    return ','.join(names[:30]) if names else '--'


def _save_profile(profile: dict):
    PROFILE_FILE.parent.mkdir(parents=True, exist_ok=True)
    PROFILE_FILE.write_text(
        json.dumps(profile, ensure_ascii=False),
        encoding='utf-8',
    )


def _do_login(p, session_file: Path):
    """直接打开 passport.goofish.com 登录页（qrCodeFirst=true），提取 QR canvas，等待扫码跳转"""
    import base64

    LOGIN_URL = (
        'https://passport.goofish.com/mini_login.htm'
        '?lang=zh_cn&appName=xianyu&appEntrance=web'
        '&styleType=vertical&qrCodeFirst=true'
        '&isMobile=false&notLoadSsoView=false&notKeepLogin=false'
        '&redirect_url=https%3A%2F%2Fwww.goofish.com%2F'
    )

    # 先通知 Tauri 弹出登录窗口（显示加载中），避免等 Chromium / 页面加载完成才出现窗口
    _emit({'status': 'need_login', 'qr_b64': ''})
    _emit_log('获取登录二维码')
    _emit_log('登录脚本启动：准备启动 Chromium')

    browser = p.chromium.launch(
        headless=False,
        args=[
            '--no-sandbox',
            '--disable-dev-shm-usage',
            '--window-position=-32000,-32000',
            '--window-size=500,600',
        ],
    )
    ctx = browser.new_context(user_agent=UA, viewport={'width': 500, 'height': 600})
    page = ctx.new_page()
    page.add_init_script(STEALTH)
    _emit_log('Chromium 已启动：准备打开登录页')

    try:
        page.goto(LOGIN_URL, wait_until='domcontentloaded', timeout=20000)
        _emit_log(f'登录页已打开：{page.url}')
    except Exception:
        _emit_log('登录页打开超时，继续尝试读取页面内容', 'warning')

    # 等待 #qrcode-img canvas 元素出现
    try:
        page.wait_for_selector('#qrcode-img canvas', timeout=10000)
        _emit_log('二维码 canvas 已出现')
    except Exception:
        _emit_log('等待二维码 canvas 超时，准备尝试页面截图兜底', 'warning')

    # 等待 canvas 尺寸正确（QR 码由 API 异步绘制，需等填充完毕）
    try:
        page.wait_for_function("""
            () => {
                const c = document.querySelector('#qrcode-img canvas');
                if (!c) return false;
                return c.width >= 100 && c.height >= 100;
            }
        """, timeout=8000)
        _emit_log('二维码 canvas 尺寸已就绪')
    except Exception:
        _emit_log('等待二维码 canvas 尺寸超时，准备尝试截图', 'warning')

    # 提取 QR canvas：优先元素截图，避免 canvas 因跨域 logo 被污染导致 toDataURL 失败
    qr_b64 = ''
    try:
        el = page.query_selector('#qrcode-img canvas')
        if el:
            qr_b64 = base64.b64encode(el.screenshot()).decode()
            _emit_log('二维码 canvas 截图成功')
    except Exception:
        _emit_log('二维码 canvas 截图失败，准备页面截图兜底', 'warning')

    # 兜底：截取页面截图
    if not qr_b64:
        try:
            qr_b64 = base64.b64encode(page.screenshot(full_page=False)).decode()
            _emit_log('登录页截图兜底成功')
        except Exception:
            _emit_log('登录页截图兜底失败', 'error')

    if not qr_b64:
        browser.close()
        _emit({'error': '咸鱼登录二维码加载失败，请重试'})
        return

    _emit({'status': 'qr_ready', 'qr_b64': qr_b64})
    _emit_log('登录二维码已生成，请扫码')
    _emit_log('等待手机扫码并确认登录')

    # 等待扫码成功：扫码、手机确认、Cookie 写入不是同一瞬间，逐步判断可避免误判。
    login_ok = False
    profile = {}
    last_verify_time = 0
    last_debug_time = 0
    scan_logged = False
    success_logged = False
    deadline = time.time() + 180
    while time.time() < deadline:
        try:
            _body_text, scan_seen, success_seen, _frame_urls = _read_login_page_signals(page)
            if scan_seen and not scan_logged:
                scan_logged = True
                _emit_log('已扫码，等待手机确认登录')
            if success_seen and not success_logged:
                success_logged = True
                _emit_log('已检测到手机确认，正在验证登录态')

            now = time.time()
            if success_seen or now - last_verify_time >= 5:
                last_verify_time = now
                login_ok, profile, verify_reason = _verify_login_context(ctx, allow_text_fallback=success_seen)
                if not login_ok and success_seen:
                    profile = profile or _collect_profile(ctx, page)
                    login_ok = True
                    verify_reason = '登录页已确认，保存当前浏览器会话'
                if login_ok:
                    _emit_log(f'登录态验证通过：{verify_reason}', 'success')
                    break
                if scan_seen and now - last_debug_time >= 10:
                    last_debug_time = now
                    profile = profile or _collect_profile(ctx, page)
                    _emit_log(
                        f'仍在等待登录态：{verify_reason}；当前 Cookie：{_format_cookie_names(profile)}',
                        'warning',
                    )
        except Exception:
            pass
        page.wait_for_timeout(1000)

    if not login_ok:
        browser.close()
        _emit({'error': '咸鱼扫码登录超时，请重试'})
        return

    session_file.parent.mkdir(parents=True, exist_ok=True)
    ctx.storage_state(path=str(session_file))
    profile = profile or _collect_profile(ctx)
    _save_profile(profile)
    browser.close()

    _emit_log(f"登录信息内容：{json.dumps(profile, ensure_ascii=False)}", 'info')
    _emit_log('扫码成功：已登录', 'success')
    _emit({'status': 'login_ok', 'username': '已登录', 'profile': profile})


def login_xianyu():
    from playwright.sync_api import sync_playwright

    with sync_playwright() as p:
        _do_login(p, SESSION_FILE)


def search_xianyu(keyword: str, category: str = ''):
    from playwright.sync_api import sync_playwright

    with sync_playwright() as p:
        _emit_log(f'商品最低价搜索：{keyword}')
        rejected_terms = _get_rejected_terms(category)
        if rejected_terms:
            _emit_log(f'已启用过滤词：{category or "cpu+gpu+memory+ssd+motherboard"}（{len(rejected_terms)} 个）')
        _prices, needs_login, links, priced_items = _run_search(p, keyword, SESSION_FILE, category)

        if needs_login:
            _emit_log('未检测到有效登录态，已停止搜索', 'warning')
            _emit({'status': 'need_login_required'})
            return None, None

    priced_items = _sort_items_by_price(priced_items)[:8]

    for item in priced_items:
        _emit_log(
            f'搜到商品链接（详情接口￥{_format_price(item.get("price"))}）：{item.get("url")}',
            'link',
            url=item.get('url'),
        )

    if not priced_items:
        for link in links[:8]:
            _emit_log(f'搜到商品链接（￥--）：{link}', 'link', url=link)

    if priced_items:
        best = priced_items[0]
        _emit_log(
            f'最终采用详情接口最低价：￥{_format_price(best["price"])}，链接：{best["url"]}',
            'success',
            url=best['url'],
        )
        return best['price'], best['url']

    if links:
        _emit_log('未从详情接口拿到有效价格，已返回商品链接但不填价格', 'warning', url=links[0])
        return None, links[0]

    return None, None


if __name__ == '__main__':
    if len(sys.argv) >= 2 and sys.argv[1] == '--login':
        try:
            login_xianyu()
        except Exception as e:
            _emit({'error': str(e)})
        sys.exit(0)

    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument('--category', default='')
    parser.add_argument('keyword', nargs='*')
    args = parser.parse_args()

    if not args.keyword:
        _emit({'error': '缺少关键词参数，用法: python xianyu_search.py [--category cpu] <关键词>'})
        sys.exit(1)

    kw = ' '.join(args.keyword)
    try:
        price, url = search_xianyu(kw, args.category)
        _emit({'xianyu': price, 'xianyu_url': url})
    except Exception as e:
        _emit({'error': str(e)})
