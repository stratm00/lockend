GET http://localhost:8001/all
HTTP 200
[Asserts]
jsonpath "$" count == 30

POST http://localhost:8001/
{
  "user": 1, "id": 1
}
HTTP 200

GET http://localhost:8001/id/1
HTTP 200
[Asserts]
jsonpath "$.state.Locked" == 1

GET http://localhost:8001/all
HTTP 200
[Asserts]
jsonpath "$[1].state.Locked" == 1


# Undo previous actuation
POST http://localhost:8001/
{
    "user": 1, "id": 1
}
HTTP 200
