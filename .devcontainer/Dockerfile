FROM ubuntu:lunar

WORKDIR /home/

COPY . .

ADD setup.sh /

RUN /setup.sh

ENV PATH="/root/.cargo/bin:$PATH"