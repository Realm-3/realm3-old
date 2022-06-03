FROM ubuntu as builder

COPY . ./realm3

RUN apt-get update
RUN apt-get install git clang curl libssl-dev llvm libudev-dev curl make -y
RUN apt-get update

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup default stable
RUN rustup update
RUN rustup update nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly

WORKDIR /realm3

RUN cargo build --release

FROM ubuntu as runner
RUN apt-get update
COPY --from=builder /realm3/target/release/realm3 /usr/local/bin/realm3

ENTRYPOINT ["realm3"]
# You should be able to run a validator using this docker image in a bash environmment with the following command:
#docker run -d \
#-p 30333:30333 \
#-p 9922:9922 \
#-p 9934:9934 \
#-v /tmp/node0:/node-temp \
#-v /root/estatetial-chain/chain-spec:/data \
#<image-name> \
#--base-path /node-temp \
#--chain ./data/customSpecRaw.json \
#--ws-port 9922 \
#--port 30333 \
#--rpc-port 9934 \
#--validator \
#--rpc-methods Unsafe \
#--ws-external \
#--rpc-external \
#--rpc-cors all \
#--name "Validator Name" \
