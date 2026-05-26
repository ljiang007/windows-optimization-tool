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
from pathlib import Path

UA = (
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) '
    'AppleWebKit/537.36 (KHTML, like Gecko) '
    'Chrome/125.0.0.0 Safari/537.36'
)
STEALTH = "Object.defineProperty(navigator,'webdriver',{get:()=>undefined})"
SESSION_FILE = Path(os.environ.get('APPDATA', '.')) / 'system_toolbox' / 'xianyu_session.json'


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


def _run_search(p, keyword: str, session_file: Path):
    """headless 搜索，返回 (价格列表, 是否需要登录)"""
    api_prices = []

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
            api_prices.extend(_parse_prices_from_response(resp.text()))
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
    browser.close()

    needs_login = not api_prices and '立即登录' in body
    return api_prices, needs_login


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
    print(json.dumps({'status': 'need_login', 'qr_b64': ''}, ensure_ascii=False), flush=True)

    browser = p.chromium.launch(headless=True, args=['--no-sandbox', '--disable-dev-shm-usage'])
    ctx = browser.new_context(user_agent=UA, viewport={'width': 500, 'height': 600})
    page = ctx.new_page()
    page.add_init_script(STEALTH)

    try:
        page.goto(LOGIN_URL, wait_until='domcontentloaded', timeout=20000)
    except Exception:
        pass

    # 等待 #qrcode-img canvas 元素出现
    try:
        page.wait_for_selector('#qrcode-img canvas', timeout=10000)
    except Exception:
        pass

    # 等待 canvas 尺寸正确（QR 码由 API 异步绘制，需等填充完毕）
    try:
        page.wait_for_function("""
            () => {
                const c = document.querySelector('#qrcode-img canvas');
                if (!c) return false;
                return c.width >= 100 && c.height >= 100;
            }
        """, timeout=8000)
    except Exception:
        pass

    # 提取 QR canvas：优先元素截图，避免 canvas 因跨域 logo 被污染导致 toDataURL 失败
    qr_b64 = ''
    try:
        el = page.query_selector('#qrcode-img canvas')
        if el:
            qr_b64 = base64.b64encode(el.screenshot()).decode()
    except Exception:
        pass

    # 兜底：截取页面截图
    if not qr_b64:
        try:
            qr_b64 = base64.b64encode(page.screenshot(full_page=False)).decode()
        except Exception:
            pass

    if not qr_b64:
        browser.close()
        print(json.dumps({'error': '咸鱼登录二维码加载失败，请重试'}, ensure_ascii=False), flush=True)
        return

    print(json.dumps({'status': 'qr_ready', 'qr_b64': qr_b64}, ensure_ascii=False), flush=True)

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
        print(json.dumps({'error': '咸鱼扫码登录超时，请重试'}, ensure_ascii=False), flush=True)
        return

    session_file.parent.mkdir(parents=True, exist_ok=True)
    ctx.storage_state(path=str(session_file))
    browser.close()

    print(json.dumps({'status': 'login_ok'}, ensure_ascii=False), flush=True)


def search_xianyu(keyword: str):
    from playwright.sync_api import sync_playwright

    with sync_playwright() as p:
        prices, needs_login = _run_search(p, keyword, SESSION_FILE)

        if needs_login:
            _do_login(p, SESSION_FILE)
            prices, _ = _run_search(p, keyword, SESSION_FILE)

    candidates = sorted(set(prices))
    return candidates[0] if candidates else None


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print(json.dumps({'error': '缺少关键词参数，用法: python xianyu_search.py <关键词>'}))
        sys.exit(1)

    kw = ' '.join(sys.argv[1:])
    try:
        price = search_xianyu(kw)
        print(json.dumps({'xianyu': price}, ensure_ascii=False))
    except Exception as e:
        print(json.dumps({'error': str(e)}, ensure_ascii=False))
