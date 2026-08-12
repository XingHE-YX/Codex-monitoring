# Codex 重置预警

这是一个 Android 优先、单用户单设备的 Codex 使用额度重置监测项目。

当前仓库阶段是“规范与实施计划已完成，代码尚未开始”。产品由 Android Kotlin/Jetpack Compose 客户端和香港服务器上的 Rust 核心服务组成。服务按北京时间每小时检查三个指定公开来源，并只对符合 A 级或 B 级规则的未来额度重置信号发送通知。

## 规范文档

- [产品需求文档](PRD.md)
- [Android 应用流程](APP_FLOW.md)
- [技术栈](TECH_STACK.md)
- [前端指南](FRONTEND_GUIDELINES.md)
- [后端结构](BACKEND_STRUCTURE.md)
- [实施计划](IMPLEMENTATION_PLAN.md)
- [代理规范](AGENT.md)

## 当前状态文件

- [实施进度](progress.txt)
- [长期经验](lessons.md)

## 开始开发前

请先阅读 `AGENT.md`，然后完整阅读 `progress.txt` 和 `lessons.md`，再根据 `IMPLEMENTATION_PLAN.md` 从任务 1.1 开始。真实凭据、Firebase 服务账号文件、Gmail 应用专用密码、私钥和构建产物不得提交到仓库。
