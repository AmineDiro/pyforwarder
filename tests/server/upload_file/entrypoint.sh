#!/bin/bash
/usr/sbin/sshd -E /app/sshd_log && python3 api_file_upload.py