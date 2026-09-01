FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
  LANG=C.UTF-8 \
  TZ=UTC

USER root

RUN apt-get update && \
  apt-get install -y curl git build-essential pkg-config libssl-dev && \
  apt-get clean && rm -rf /var/lib/apt/lists/*

RUN curl -L -o /tmp/sunspot.deb "https://github.com/reilabs/sunspot/releases/download/v1.0.0/sunspot_1.0.0_linux_amd64.deb" \
  && apt-get -y install /tmp/sunspot.deb \
  && rm /tmp/sunspot.deb

USER ubuntu
WORKDIR /home/ubuntu

RUN echo 'export GNARK_VERIFIER_BIN="$HOME/sunspot/gnark-solana/crates/verifier-bin"' >> /home/ubuntu/.bashrc

RUN curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
ENV PATH="/home/ubuntu/.local/share/solana/install/active_release/bin:${PATH}"

RUN curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash
ENV PATH="/home/ubuntu/.noirup/bin:/home/ubuntu/.nargo/bin:${PATH}"
RUN noirup

RUN mkdir /home/ubuntu/workspace

WORKDIR /home/ubuntu/workspace

CMD ["/bin/bash"]
