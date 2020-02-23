# Korat

## Notice

I'm no longer maintaining this project (I haven't for a few years in fact. Dynomite is a different lib which was initially based on korat.

## Intro

Korat is a library for the creation of convenience methods when working with dynamodb items

Korat provides attribute converters for most supported types from rusoto AttributeValue. It
also provides a deriveable implementation for try_from and from converters for your structs
(to and from rusoto AttributeMap).

This is work in progress. 

# Serializables

All items implementing the DynamoDBItem trait are serializable and can be stored into
DynamoDB tables or as fields within other items

# Insertables

As a convenience method for DynamoDBItems you can implement the trait DynamoDBInsertable
which provides methods for accessing the keys of the object. This makes it possible to get
the keys of an existing item.

In addition to this, when deriving the implementations, a new "Key" struct will be
automatically implemented for your types which will allow you to easily create the keys for
fetching and querying your DynamoDB tables.
