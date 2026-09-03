FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
  LANG=C.UTF-8 \
  TZ=UTC

USER root

RUN apt-get update && \
  apt-get install -y curl git build-essential pkg-config libssl-dev ca-certificates --no-install-recommends && \
  apt-get clean && rm -rf /var/lib/apt/lists/* && \
  curl -L -o /tmp/sunspot.deb "https://github.com/reilabs/sunspot/releases/download/v1.0.0/sunspot_1.0.0_linux_amd64.deb" \
  && apt-get -y install /tmp/sunspot.deb \
  && rm /tmp/sunspot.deb

USER ubuntu
WORKDIR /home/ubuntu

ENV PATH="/home/ubuntu/.local/share/solana/install/active_release/bin:/home/ubuntu/.cargo/bin:/home/ubuntu/.avm/bin:/home/ubuntu/.noirup/bin:/home/ubuntu/.nargo/bin:${PATH}"
ENV NVM_DIR="/home/ubuntu/.nvm"

RUN echo 'export GNARK_VERIFIER_BIN="$HOME/workspace/sunspot/gnark-solana/crates/verifier-bin"' >> /home/ubuntu/.bashrc && \
  curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash && \
  curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash && \
  noirup && \
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.7/install.sh | METHOD=script bash && \
  bash -c "source $NVM_DIR/nvm.sh && nvm install node && npm install -g yarn" && \
  echo 'export NVM_DIR="$HOME/.nvm"' >> $HOME/.bashrc && \
  echo '[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm' >> $HOME/.bashrc && \
  echo '[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion' >> $HOME/.bashrc && \
  mkdir /home/ubuntu/workspace

WORKDIR /home/ubuntu/workspace

CMD ["/bin/bash"]
