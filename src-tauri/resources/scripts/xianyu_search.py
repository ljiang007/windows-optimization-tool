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
from urllib.parse import unquote
from pathlib import Path

UA = (
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) '
    'AppleWebKit/537.36 (KHTML, like Gecko) '
    'Chrome/125.0.0.0 Safari/537.36'
)
STEALTH = "Object.defineProperty(navigator,'webdriver',{get:()=>undefined})"
SESSION_FILE = Path(os.environ.get('APPDATA', '.')) / 'system_toolbox' / 'xianyu_session.json'
PROFILE_FILE = Path(os.environ.get('APPDATA', '.')) / 'system_toolbox' / 'xianyu_profile.json'


def _emit(payload: dict):
    print(json.dumps(payload, ensure_ascii=True), flush=True)


def _emit_log(message: str, level: str = 'info', **extra):
    payload = {'status': 'log', 'type': level, 'message': message}
    payload.update(extra)
    _emit(payload)


def _parse_prices_from_response(raw_json: str) -> list:
    results = []
    for m in re.findall(
        r'"(?:price|soldPrice|currentPrice|originalPrice)"\s*:\s*"?(\d+(?:\.\d+)?)"?',
        raw_json,
    ):
        try:
            v = float(m)
            if 1 < v < 999999:
                results.append(v)
        except ValueError:
            pass
    return results


def _parse_links_from_response(raw_json: str) -> list:
    links = []
    for raw in re.findall(r'https?:\\?/\\?/[^"\\]+', raw_json):
        url = _normalize_item_url(raw)
        if url:
            links.append(url)

    for item_id in re.findall(r'"(?:itemId|item_id|id)"\s*:\s*"?(\d{6,})"?', raw_json):
        links.append(f'https://www.goofish.com/item?id={item_id}')

    return _unique(links)


def _normalize_item_url(value: str) -> str:
    if not value:
        return ''
    url = str(value).replace('\\/', '/').strip()
    if url.startswith('//'):
        url = f'https:{url}'
    if 'goofish.com' not in url or ('item' not in url and 'detail' not in url):
        return ''
    return url


def _parse_price(value):
    if value is None:
        return None
    if isinstance(value, (int, float)):
        price = float(value)
    else:
        match = re.search(r'(\d+(?:\.\d+)?)', str(value).replace(',', ''))
        if not match:
            return None
        price = float(match.group(1))
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


def _run_search(p, keyword: str, session_file: Path):
    """headless 搜索，返回 (价格列表, 是否需要登录, 商品链接列表, 带价格商品列表)"""
    api_prices = []
    item_links = []
    priced_items = []

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
            api_prices.extend(_parse_prices_from_response(raw))
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

    body = page.inner_text('body')
    try:
        dom_links = page.eval_on_selector_all(
            'a[href]',
            """
            els => els.map((a) => {
              try { return new URL(a.getAttribute('href'), location.href).href } catch { return '' }
            }).filter((href) => href.includes('goofish.com') && (href.includes('item') || href.includes('detail')))
            """,
        )
        item_links.extend(dom_links)
    except Exception:
        pass
    try:
        dom_items = page.eval_on_selector_all(
            'a[href]',
            """
            (els, keyword) => {
              const normalize = (href) => {
                try { return new URL(href, location.href).href } catch { return '' }
              };
              const normalizeText = (text) => (text || '').replace(/\\s+/g, '').toUpperCase();
              const keywordText = normalizeText(keyword);
              const isVisible = (el) => {
                const rect = el.getBoundingClientRect();
                const style = window.getComputedStyle(el);
                return rect.width > 0 && rect.height > 0 && style.display !== 'none' && style.visibility !== 'hidden';
              };
              const findPrice = (text) => {
                if (!text) return null;
                const matches = [...text.matchAll(/(?:¥|￥)\\s*(\\d+(?:\\.\\d+)?)/g)]
                  .map((m) => Number(m[1]))
                  .filter((n) => Number.isFinite(n) && n > 1 && n < 999999);
                return matches.length ? Math.min(...matches) : null;
              };
              const isRejectedItem = (text) => {
                const rejectedTerms = ['坏件', '尸体', '点不亮', '不亮', '故障', '报废', '钥匙扣', '练手', '空盒', '包装盒'];
                return rejectedTerms.some((term) => text.includes(term));
              };
              const findCardPrice = (link) => {
                let node = link;
                for (let i = 0; i < 6 && node; i += 1) {
                  if (!isVisible(node)) {
                    node = node.parentElement;
                    continue;
                  }
                  const rect = node.getBoundingClientRect();
                  const text = node.innerText || node.textContent || '';
                  const price = findPrice(text);
                  const textOk = text.length > 0 && text.length < 900;
                  const sizeOk = rect.width >= 120 && rect.height >= 60 && rect.height <= 520 && rect.width <= window.innerWidth + 80;
                  const keywordOk = !keywordText || normalizeText(text).includes(keywordText);
                  if (price != null && textOk && sizeOk && keywordOk && !isRejectedItem(text)) return price;
                  node = node.parentElement;
                }
                return null;
              };
              const seen = new Set();
              const results = [];
              for (const a of els) {
                const url = normalize(a.getAttribute('href') || '');
                if (!url.includes('goofish.com') || (!url.includes('item') && !url.includes('detail'))) continue;
                const price = findCardPrice(a);
                if (price != null && !seen.has(url)) {
                  seen.add(url);
                  results.push({ url, price });
                }
              }
              return results;
            }
            """,
            keyword,
        )
        priced_items.extend(_sort_items_by_price(dom_items or [])[:8])
    except Exception:
        pass
    browser.close()

    needs_login = not api_prices and not priced_items and '立即登录' in body
    return api_prices, needs_login, _unique(item_links), _unique_items(priced_items)


def _collect_profile(ctx, page=None) -> dict:
    candidate_names = ('tracknick', '_nk_', 'lgc', 'sn', 'unb')
    cookies = ctx.cookies()
    candidates = {}
    for cookie in cookies:
        name = cookie.get('name')
        if name in candidate_names:
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

    browser = p.chromium.launch(headless=True, args=['--no-sandbox', '--disable-dev-shm-usage'])
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

    # 等待扫码成功：必须实际访问咸鱼首页验证不再显示「立即登录」，避免游客 cookie 误判
    login_ok = False
    deadline = time.time() + 180
    while time.time() < deadline:
        try:
            body_text = ''
            try:
                body_text = page.inner_text('body')
            except Exception:
                pass
            maybe_confirmed = (
                '扫码成功' in body_text
                or '登录成功' in body_text
                or '已确认' in body_text
                or '授权成功' in body_text
                or 'www.goofish.com' in page.url
            )
            if maybe_confirmed:
                try:
                    verify_page = ctx.new_page()
                    verify_page.add_init_script(STEALTH)
                    verify_page.goto('https://www.goofish.com/', wait_until='domcontentloaded', timeout=15000)
                    verify_page.wait_for_timeout(1500)
                    verify_text = verify_page.inner_text('body')
                    login_ok = '立即登录' not in verify_text
                    profile = _collect_profile(ctx, verify_page)
                    verify_page.close()
                    if login_ok:
                        break
                except Exception:
                    pass
        except Exception:
            pass
        page.wait_for_timeout(1000)

    if not login_ok:
        browser.close()
        _emit({'error': '咸鱼扫码登录超时，请重试'})
        return

    session_file.parent.mkdir(parents=True, exist_ok=True)
    ctx.storage_state(path=str(session_file))
    profile = locals().get('profile') or _collect_profile(ctx)
    _save_profile(profile)
    browser.close()

    _emit_log(f"登录信息内容：{json.dumps(profile, ensure_ascii=False)}", 'info')
    _emit_log('扫码成功：已登录', 'success')
    _emit({'status': 'login_ok', 'username': '已登录', 'profile': profile})


def login_xianyu():
    from playwright.sync_api import sync_playwright

    with sync_playwright() as p:
        _do_login(p, SESSION_FILE)


def search_xianyu(keyword: str):
    from playwright.sync_api import sync_playwright

    with sync_playwright() as p:
        _emit_log(f'商品最低价搜索：{keyword}')
        prices, needs_login, links, priced_items = _run_search(p, keyword, SESSION_FILE)

        if needs_login:
            _emit_log('未检测到有效登录态，已停止搜索', 'warning')
            _emit({'status': 'need_login_required'})
            return None, None

    priced_items = _sort_items_by_price(priced_items)[:8]

    for item in priced_items:
        _emit_log(
            f'搜到商品链接（￥{_format_price(item.get("price"))}）：{item.get("url")}',
            'link',
            url=item.get('url'),
        )

    if not priced_items:
        for link in links[:8]:
            _emit_log(f'搜到商品链接（￥--）：{link}', 'link', url=link)

    if priced_items:
        best = priced_items[0]
        _emit_log(
            f'最终采用最低价：￥{_format_price(best["price"])}，链接：{best["url"]}',
            'success',
            url=best['url'],
        )
        return best['price'], best['url']

    if links:
        return None, links[0]

    candidates = sorted(set(prices))
    return (candidates[0] if candidates else None), None


if __name__ == '__main__':
    if len(sys.argv) >= 2 and sys.argv[1] == '--login':
        try:
            login_xianyu()
        except Exception as e:
            _emit({'error': str(e)})
        sys.exit(0)

    if len(sys.argv) < 2:
        _emit({'error': '缺少关键词参数，用法: python xianyu_search.py <关键词>'})
        sys.exit(1)

    kw = ' '.join(sys.argv[1:])
    try:
        price, url = search_xianyu(kw)
        _emit({'xianyu': price, 'xianyu_url': url})
    except Exception as e:
        _emit({'error': str(e)})
