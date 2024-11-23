| impl trait                                         | api                             |
|----------------------------------------------------|---------------------------------|
| trait Actor1                                       |                                 |
| fn inc(&mut self, data: u8) -> ActorResult<>       | let **mut** actor = ...         |
|                                                    | actor.inc(2).unwrap()           |
| async fn inc(&mut self, data: u8) -> ActorResult<> | let **mut** actor = ...         |
|                                                    | actor.inc(2).**await**.unwrap() |

| impl mut / trait                             | generated                                          | api                     | Result |
|----------------------------------------------|----------------------------------------------------|-------------------------|--------|
| trait Actor1                                 | trait Actor1Mut                                    |                         | --     |
| fn inc(&self, data: u8) -> ActorResult<>     | fn inc(&mut self, data: u8) -> ActorResult<>       | let actor: Actor1 = ... | --     |
|                                              |                                                    | actor.inc(2).unwrap()   | --     |
| fn inc(&mut self, data: u8) -> ActorResult<> | async fn inc(&mut self, data: u8) -> ActorResult<> | let actor: Actor1 = ... | --     |
|                                              |                                                    | actor.inc(2).unwrap()   | --     |

| impl mut / trait | generated       | 
|------------------|-----------------|
| trait            | trait Actor1Mut |