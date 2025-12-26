FROM summerwind/actions-runner:v2.283.3-ubuntu-20.04-b01e193 as actions-runner

WORKDIR /starcoin
COPY rust-toolchain.toml /starcoin/rust-toolchain.toml
COPY scripts/dev_setup.sh /starcoin/scripts/dev_setup.sh

# Upgrade docker client version using apt (only update Docker repository)
RUN sudo apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        gnupg \
        lsb-release && \
    sudo mkdir -p /etc/apt/keyrings && \
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null && \
    (sudo apt-get update || true) && \
    sudo apt-get install -y --no-install-recommends docker-ce-cli && \
    sudo apt-get clean && \
    sudo rm -rf /var/lib/apt/lists/* && \
    docker --version

RUN /starcoin/scripts/dev_setup.sh -b -t -y -o -p && \
  sudo apt-get clean && \
  sudo rm -rf /var/lib/apt/lists/*

ENV DOTNET_ROOT "${HOME}/.dotnet"
ENV Z3_EXE "${HOME}/bin/z3"
#ENV CVC4_EXE "${HOME}/bin/cvc4"
ENV CVC5_EXE "${HOME}/bin/cvc5"
ENV BOOGIE_EXE "${DOTNET_ROOT}/tools/boogie"
ENV PATH "${HOME}/.cargo/bin:/usr/lib/golang/bin:${HOME}/bin:${DOTNET_ROOT}:${DOTNET_ROOT}/tools:$PATH"