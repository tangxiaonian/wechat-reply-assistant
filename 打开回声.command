#!/bin/zsh
# Open the locally built preview without installing anything or modifying WeChat.
TASK_ROOT="$(cd -- "$(dirname -- "$0")" && pwd)"
TASK_APP="$TASK_ROOT/src-tauri/target/debug/bundle/macos/回声.app"
if [[ -d "$TASK_APP" ]]; then
  open "$TASK_APP"
else
  printf '%s\n' '尚未找到体验版应用，请先按 README 中的命令完成本机打包。'
  read -r '?按回车关闭…'
fi
