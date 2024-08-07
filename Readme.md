# MQTT Relay

A simple service that can be used to pusblish MQTT messages when others are received.

## Configuring the service itself

See `config.example.yaml` for all configuration options.

## Configuring MQTT message mappings

Mappings between topics are managed by creating files, one for each topic the service should subscribe to.

This first examplle file will result in the service subscribing to the MQTT topic `my/topic` (based on its directory and file name). When a message on the topic is received, the service will publish a messagee to `other/topic` with the given payload.

```yaml
# ./mappings/my/topic.yaml
messages:
  - topic: "other/topic"
    message: |-
      { "data": "just a string" }
```

To send multiple messages when a single one is recieved:

```yaml
# ./mappings/my/topic.yaml
messages:
  - topic: "other/topic1"
    message: |-
      { "info": "first message" }
  - topic: "other/topic2"
    message: |-
      { "info": "second message" }
```

If you need to listen to multiple topics, create multiple files.
This example will subscribe to `my/first/topic` and `my/second/topic`:

```yaml
# ./mappings/my/first/topic.yaml
messages:
  - topic: "other/topic1"
    message: |-
      { "info": "first message" }
```
```yaml
# ./mappings/my/second/topic.yaml
messages:
  - topic: "other/topic1"
    message: |-
      { "info": "second message" }
```

You can attatch conditions to messages. The service will evaluate the condition using `jq` and only send the messages if it evaluates to `"true"`.

In the following example, the message will only be sent if the received message inclues a JSON that contains `{"hello": "world"}`:

```yaml
# ./mappings/my/topic.yaml
conditions:
  my_condition_name: .hello == "world"

messages:
  - condition: "my_condition_name"
    topic: "other/topic"
    message: |-
      { "data": "just a string" }
```
