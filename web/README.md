# Auto-NovelAI WebUI (Vite + Vue + TailwindCSS v4 + daisyUI)

## 开发启动

1. 先启动后端（默认监听 127.0.0.1:11451）

2. 再启动前端：

```bash
cd webui
npm install
npm run dev
```

## 后端地址

- 默认使用 `http://127.0.0.1:11451`
- 也可以通过环境变量覆盖：复制 `.env.example` 为 `.env` 并修改 `VITE_BACKEND_URL`

Vite dev server 会把 `/api/*` 与 `/outputs/*` 代理到后端。
