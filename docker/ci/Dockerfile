FROM summerwind/actions-runner:v2.283.3-ubuntu-20.04-b01e193 as actions-runner

WORKDIR /starcoin
COPY rust-toolchain /starcoin/rust-toolchain
COPY scripts/dev_setup.sh /starcoin/scripts/dev_setup.sh

RUN /starcoin/scripts/dev_setup.sh -b -t -y -o -p && \
  sudo apt-get clean && \
  sudo rm -rf /var/lib/apt/lists/*

ENV DOTNET_ROOT "${HOME}/.dotnet"
ENV Z3_EXE "${HOME}/bin/z3"
#ENV CVC4_EXE "${HOME}/bin/cvc4"
ENV CVC5_EXE "${HOME}/bin/cvc5"
ENV BOOGIE_EXE "${DOTNET_ROOT}/tools/boogie"
ENV PATH "${HOME}/.cargo/bin:/usr/lib/golang/bin:${HOME}/bin:${DOTNET_ROOT}:${DOTNET_ROOT}/tools:$PATH"