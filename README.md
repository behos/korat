# korat

Korat is a library for the creation of convenience methods when working with dynamodb items

Korat provides attribute converters for most supported types from rusoto AttributeValue. It also provides a deriveable implementation for
try_from and from converters for your structs (to and from rusoto AttributeMap).

This is work in progress. At the moment the deserializers are usable but the serializers are coming up
