FROM ghcr.io/alchemy-studio-robot/htybasic
COPY htycommons ./htycommons
COPY htyuc ./htyuc
RUN cd htyuc; cargo build --release

FROM debian:buster
# copy exe file
COPY --from=0 /htyuc/target/release/htyuc /root/htyuc
# copy scripts file
COPY htyuc/container/*.sh /root/
COPY htyuc/config/log4rs.yaml /root/config/
ARG OPERATOR
ARG OPERATOR_PWD
ARG FORM_API_KEY
ENV OPERATOR=$OPERATOR
ENV OPERATOR_PWD=$OPERATOR_PWD
ENV FORM_API_KEY=$FORM_API_KEY
RUN apt-get update -y
RUN apt-get install -y libpq-dev
RUN /root/setup.sh
ENTRYPOINT /root/init.sh
