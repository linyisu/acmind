# 添加新 parser

新增一个 OJ 解析器只需要：

1. 在本目录新建 `<oj>.js`
2. 用 `defineParser({...})` 默认导出
3. 重新 `pnpm build`

构建脚本会自动扫描本目录（除 `base.js` / `registry.js` / `_generated.js` 外）并把所有 parser 注册进 `scraper-bridge.js`。

用户点扩展图标 = 执行 `parser.pickMode(pageType)` 选出来的 mode，没有 popup。

## 最小示例

```js
import { defineParser } from "./base.js";

export default defineParser({
  name: "codeforces",
  displayName: "Codeforces",

  matches: (url) => /^https?:\/\/codeforces\.com\//.test(url),

  detectPageType: (url) => {
    if (/\/problem\//.test(url)) return "problem";
    return "other";
  },

  // 决定点击图标时该跑哪个 mode。返回 null = 此页面不支持导入。
  pickMode: (pageType) => (pageType === "problem" ? "problem-full" : null),

  modes: {
    "problem-full": {
      label: "题目 + 全部提交",  // 顶部进度条旁的简短说明
      scrape: async ({ progress, url, document }) => {
        progress({ message: "正在抓取题面", pct: 10 });
        // ...
        progress({ message: "正在抓取提交", pct: 60 });
        // ...
        return { type: "problem-full", problem, submissions };
      },
    },
  },
});
```

## `progress` 协议

```js
progress("仅文字更新")           // 等价于 { message: "仅文字更新" }
progress({ pct: 35 })            // 仅推进进度条
progress({ message: "...", pct: 80 })  // 同时更新
```

`pct` 是 0-100 的累计百分比，框架内部只允许单调上升，所以传 50 之后再传 30 不会让进度条回退。

**约定**：parser 自己不要推进度到 100。把 95~100 留给后续上传/落库阶段，整个流程的 100% 只会在最末尾命中一次，这样进度条不会中途"重置成新一条"。

## scrape 上下文

- `progress(input)` —— 推送进度（见上）
- `url` —— `window.location.href` 副本
- `document` —— 页面 DOM，跑在 main world，有 same-origin cookies

返回的 `payload` 必须有 `type` 字段，由 background 的 `importers` 表路由到对应的 ACMind API 端点（参见 `src/background.js`）。新加一个 type 时记得在 background 那里补一条对应的上传逻辑。
