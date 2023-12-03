FROM ubuntu:jammy
RUN apt-get update && apt-get install -y openssh-server python3
RUN mkdir -p /run/sshd

RUN useradd -m alice && echo 'alice:alicealice' | chpasswd

RUN useradd -m amine -p '*'
COPY amine_key /home/amine/.ssh/authorized_keys
RUN chmod 0644 /home/amine/.ssh/authorized_keys

RUN ssh-keygen -q -N "" -t ecdsa -b 384 -f /etc/ssh/ssh_host_ecdsa_384_key

COPY sshd_config /etc/ssh/sshd_config

WORKDIR /app
# entrypoint
COPY entrypoint.sh /scripts/entrypoint.sh
RUN chmod +x /scripts/entrypoint.sh
ENTRYPOINT ["/bin/bash", "/scripts/entrypoint.sh"]