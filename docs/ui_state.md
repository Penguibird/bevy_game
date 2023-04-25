UI State diagram

```mermaid
stateDiagram
state "Build defensive" as BD
state "Build resource" as BR

Pan --> BD
Pan --> BR
Pan --> Demolish

BD --> BR
BD --> Pan
BD --> Demolish


BR --> BD
BR --> Pan
BR --> Demolish

Demolish --> BD
Demolish --> Pan
Demolish --> BR

state "Choose defensive building" as CD
BD --> CD

state "Choose defensive building" as CR
BR --> CR
```