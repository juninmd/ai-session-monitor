pnpm install --frozen-lockfile
pnpm dev            # interface no navegador com backend simulado: http://127.0.0.1:1420/?mock=demo
pnpm tauri dev      # o app de verdade (bandeja + flyout)
pnpm verify         # lint, tipos, testes e build da interface
cargo test --workspace
