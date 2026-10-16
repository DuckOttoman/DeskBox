# DeskBox 市场服务器

公网目录： https://47.85.175.203/catalog.json
健康检查： https://47.85.175.203/health

服务器仅公开健康检查、已校验目录和 packages/*.deskbox-plugin。不提供上传接口、目录浏览或远程执行。

- 程序：/opt/deskbox-market/market.py
- 公开资源：/var/lib/deskbox-market/public
- 低权限账户：deskbox-market
- HTTPS：systemd socket 绑定 443，Python 服务以低权限账户接收 socket。
- 证书：Let’s Encrypt shortlived IP 证书；每日 03:25 由 deskbox-market-renew.timer 检查续期。renew.sh 成功更新后重启服务；使用 HTTP-01 续期，因此云安全组需长期允许 80 和 443 入站。80 平时没有常驻服务。
- 证书与 ACME 私钥只保留在服务器，不能收入源码和更新包。

维护：systemctl status deskbox-market；journalctl -u deskbox-market；systemctl list-timers deskbox-market-renew.timer。
发布：先通过打包工具验证包，按版本不可变文件名上传 packages，核对 SHA256，再将完整新目录写入临时文件并用同目录 rename 原子替换 catalog.json。公开资源由 root 持有，服务账户只读。更新不要覆盖旧版本包。
停止服务：systemctl disable --now deskbox-market.socket deskbox-market.service deskbox-market-renew.timer。不会修改 SSH 或宝塔面板。

验证：python tests/test_market.py；curl --fail https://47.85.175.203/health；curl --fail https://47.85.175.203/catalog.json。首次部署已成功签发证书并在服务器外校验 HTTPS；尚未模拟跨数日续期。
