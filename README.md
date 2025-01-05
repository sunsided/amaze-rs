# A maze generator

Example usage

```bash
cargo run -- gen --seed 1337 --width 16 --height 16 --style thin
```

Example output:

```text
╷╶───┬╴┌─┬╴┌─┐┌┐
└───┐├┐│╷└─┴╴└┘│
╶─┬┐│╵└┤├┐┌───┐│
┌─┘╵└─┐└┘│└┐┌┐└┤
└─┐┌──┘╶┬┘┌┤╵└─┘
┌┐│└───┐│┌┘│┌──┐
│└┘┌┐┌┐││└┐╵│┌┐│
├──┘└┘└┘└┐└─┘│││
└┐╷┌─┬┐┌┐└┐┌─┘└┤
╷│└┘┌┘└┘│┌┘│┌┐╶┘
│└─┐└─┐╶┘│┌┘││┌┐
├┬┐└─┐│┌─┘└─┘├┘│
│││┌┐│││┌┐╷┌┐│┌┘
││╵│└┘└┴┘└┤││╵└┐
│└┐└┐┌───┐╵│└──┤
└╴└─┴┘╶──┴─┘╶──┘
```
