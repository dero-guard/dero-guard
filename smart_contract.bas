// https://github.com/dero-guard/dero-guard
// Litarvan & Slixe

// Must be called only one-time.
Function Initialize() Uint64
1 IF EXISTS("owner") RETURN 1
2 STORE("owner", SIGNER())
10 STORE("total", 0)
11 RETURN 0
End Function

// Use this entrypoint to register yourself as an exit node provider.
Function Register(price Uint64, name String, country String) Uint64
1 IF EXISTS(addr) THEN RETURN 1
2 IF DEROVALUE() != 10000 THEN RETURN 1 // ask for 1 DERO as fee to prevent spam / useless providers
3 IF STRLEN(name) < 6 THEN RETURN 1 // minimum 6 characters for provider name
4 IF STRLEN(name) > 16 THEN RETURN 1 // maxmimum 16 characters
5 IF STRLEN(country) != 2 THEN RETURN 1 // FR / EN / US / DE etc..
3 DIM exist As Uint64
4 LET exist = EXISTS(addr)
5 DIM current As Uint64
6 IF exist THEN GOTO 8
7 LET current = LOAD("total")
10 STORE(current + ":p", price)
11 STORE(current + ":n", name)
12 STORE(current + ":c", country)
13 STORE(current + ":a", SIGNER())
14 STORE(current + ":rt", 0) // total rate
15 STORE(addr, current) // save current ID for this address
16 IF exist THEN GOTO 30
17 STORE("total", current + 1)
18 RETURN 0
End Function

// A provider can update his data on the SC
Function Update(price Uint64, name String, country String) Uint64
1 IF !EXISTS(addr) THEN RETURN 1
2 DIM id as Uint64
3 LET id = LOAD(addr)
4 IF price != 0 THEN STORE(id + ":p", price) // set the new price if its to be updated
5 IF STRLEN(name) > 16 THEN RETURN 1 // maxmimum 16 characters
6 IF STRLEN(name) > 5 THEN STORE(id + ":n", name) // set the new name
7 IF STRLEN(country) == 2 THEN STORE(id + ":c", country)
End Function

// Allows anyone to rate a provider.
Function Note(value Uint64, provider Uint64) Uint64
1 IF value < 1 AND value > 5 THEN RETURN 1
2 IF !EXISTS(provider + ":rt") THEN RETURN 1
3 DIM id as Uint64
4 LET id = LOAD(provider + ":rt")
5 IF EXISTS(provider + ":r:" + SIGNER()) THEN RETURN 1 // can only rate one-time (TODO)
6 STORE(provider + ":r:" + SIGNER(), id) // keep history
7 STORE(provider + ":r:" + id, value) // score
8 STORE(provider + ":rt", id + 1)
9 RETURN 0
End Function

// admin entrypoint to update SC code for any update
Function SCUpdate(code String) Uint64
1 IF LOAD("owner") != SIGNER() RETURN 1
2 UPDATE_SC_CODE(code)
3 RETURN 0
End Function