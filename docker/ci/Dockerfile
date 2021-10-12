FROM summerwind/actions-runner:v2.283.3-ubuntu-20.04-b01e193 as actions-runner

WORKDIR /starcoin
COPY rust-toolchain /starcoin/rust-toolchain
COPY scripts/dev_setup.sh /starcoin/scripts/dev_setup.sh

RUN /starcoin/scripts/dev_setup.sh -b -t -y -o -p && \
  apt-get clean && \
  sudo rm -rf /var/lib/apt/lists/*