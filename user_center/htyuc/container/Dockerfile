FROM debian:buster
COPY htyuc/target/release/htyuc /root/htyuc
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
