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
    """弹出可见浏览器，等用户登录咸鱼后保存 session"""
    print(json.dumps({'status': 'need_login', 'msg': '请在弹出的浏览器中扫码/登录咸鱼，完成后脚本自动继续'}, ensure_ascii=False), flush=True)

    SESSION_FILE.parent.mkdir(parents=True, exist_ok=True)
    browser = p.chromium.launch(headless=False, args=['--no-sandbox'])
    ctx = browser.new_context(user_agent=UA)
    page = ctx.new_page()
    page.add_init_script(STEALTH)
    page.goto('https://www.goofish.com/', wait_until='domcontentloaded', timeout=20000)

    # 等待「立即登录」按钮消失，即代表用户已完成登录（最多 3 分钟）
    try:
        page.wait_for_selector('text=立即登录', timeout=10000)
    except Exception:
        pass
    try:
        page.wait_for_function(
            "() => !document.body.innerText.includes('立即登录')",
            timeout=180000,
        )
    except Exception:
        pass

    page.wait_for_timeout(2000)
    ctx.storage_state(path=str(session_file))
    browser.close()


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
