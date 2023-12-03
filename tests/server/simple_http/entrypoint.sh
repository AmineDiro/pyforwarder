#!/bin/bash
/usr/sbin/sshd -E /app/sshd_log && python3 -m http.server 8181