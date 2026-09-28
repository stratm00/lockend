# Lockend

Lockend is a simple two-service containerised event sourcing-based backend that simulates a locker room

## Architecture

The backend is comprised of a postgres database and an API server.
The API server keeps a connection to the database and an image of the current locker arrangement.

## Endpoints

| HTTP Verb | URL                    | Request Data | Description                                   |
|-----------|------------------------|--------------|-----------------------------------------------|
| POST      | /                      | JSON payload | Send an event                                 |
| GET       | /all                   | None         | View the State of all lockers                 |
| GET       | /(id)                  | None         | View the State of a single locker             |
| GET       | /time_travel_id/(n)    | None         | View the state of all lockers after event #N  |
| GET       | /timestamp_travel/(ts) | None         | View the state of all lockers at timestamp ts |

## Anatomy of an event

The JSON payload for an event has the shape of:
```json 
{
  user: number,
  locker: number 
}
```

Upon successful parsing, an Event is inserted into the `events` table of the database, where it receives a run-on `cmdid` and the current timestamp.

## State Anatomy 

The in-memory state of the locker room is modeled as a fixed-size array of locker objects.

## State Updates

After a successful insertion into the database, another query is sent to `NOTIFY` a predetermined channel. This channel is listened on by the backend again. This achieves the separation of the insertion process from the update process.

Once the client receives the notification, it fetches the most recently inserted command and parses it.
This schema makes it possible to have two APIs running in parallel without state ever diverging, because the database acts as a single source of truth.

## Command Parsing

The only Command this system recognises is a metaphorical actuation of the lock , described as `LockerRoomCommand::Actuate`. The state object in API server memory takes this command and redirects it to an internal locker state object, taking only the user information withininto a `LockerCommand::Actuate`.

## States of a Locker

The state of a locker can either be unlocked or locked by a specific user.
