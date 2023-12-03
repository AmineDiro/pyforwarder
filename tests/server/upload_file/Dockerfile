FROM ubuntu:jammy
RUN apt-get update && apt-get install -y openssh-server python3 python3-pip
RUN mkdir -p /run/sshd

# Add users
RUN useradd -m alice && echo 'alice:alicealice' | chpasswd
RUN useradd -m amine -p '*'

# Setup SSH config
COPY amine_key /home/amine/.ssh/authorized_keys
RUN chmod 0644 /home/amine/.ssh/authorized_keys
RUN ssh-keygen -q -N "" -t ecdsa -b 384 -f /etc/ssh/ssh_host_ecdsa_384_key
COPY sshd_config /etc/ssh/sshd_config

# Settup python/fastapi
WORKDIR /app

RUN python3 -m pip install uvicorn[standard] fastapi python-multipart
COPY api_file_upload.py /app/api_file_upload.py

# entrypoint
COPY entrypoint.sh /scripts/entrypoint.sh
RUN chmod +x /scripts/entrypoint.sh
ENTRYPOINT ["/bin/bash", "/scripts/entrypoint.sh"]