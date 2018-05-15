# Todo

- Add support for show command to set a time range (from/to)
- Show command should allow to aggregate events into i.e. minutes
- Show command client config: color defaults and changed values

- Add carrier refresher failed to error events
- Parse client config as part of the client init
- Expose those two in the CLI config

- Aggregate stack traces into one log
- Gather those into exception events and report 
  (i.e. grouped by exception type and/or type of op)

- Aggregate infos on stuff stuck in retry loops and report