#!/bin/bash
/usr/sbin/sshd -E /app/sshd_log && python3 async_http_server.py