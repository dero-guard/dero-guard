// https://github.com/dero-guard/dero-guard
// Litarvan & Slixe

// Must be called only one-time.
Function Initialize() Uint64
1 IF EXISTS("owner") THEN GOTO 5
2 STORE("owner", SIGNER())
3 STORE("total", 0)
4 RETURN 0
5 RETURN 1
End Function

// Use this entrypoint to register or update yourself as an exit node provider.
Function Provider(price Uint64, name String, country String) Uint64
1 IF DEROVALUE() != 10000 THEN GOTO 21 // TODO ask for 1 DERO as fee to prevent spam / useless providers
2 IF STRLEN(name) < 6 THEN GOTO 21 // minimum 6 characters for provider name
3 IF STRLEN(name) > 16 THEN GOTO 21 // maxmimum 16 characters
4 IF STRLEN(country) != 2 THEN GOTO 21 // FR / EN / US / DE etc..
5 DIM exist As Uint64
6 LET exist = EXISTS(SIGNER())
7 DIM current As Uint64
8 IF exist THEN GOTO 11
9 LET current = LOAD("total")
10 GOTO 12
11 LET current = LOAD(SIGNER())
12 STORE("" + current + ":p", price)
13 STORE("" + current + ":n", name)
14 STORE("" + current + ":c", country)
15 IF exist THEN GOTO 20
16 STORE("" + current + ":a", ADDRESS_STRING(SIGNER()))
17 STORE("" + current + ":rt", 0) // total rate
18 STORE(SIGNER(), current) // save current ID for this address
19 STORE("total", current + 1)
20 RETURN 0
21 RETURN 1
End Function

// Allows anyone to rate a provider.
Function Note(value Uint64, provider Uint64) Uint64
1 IF value < 1 AND value > 5 THEN GOTO 10
2 IF !EXISTS("" + provider + ":rt") THEN GOTO 10
3 DIM id as Uint64
4 LET id = LOAD("" + provider + ":rt")
5 IF EXISTS("" + provider + ":r:" + SIGNER()) THEN GOTO 10 // can only rate one-time (TODO)
6 STORE("" + provider + ":r:" + SIGNER(), id) // keep history
7 STORE("" + provider + ":r:" + id, value) // score
8 STORE("" + provider + ":rt", id + 1)
9 RETURN 0
10 RETURN 1
End Function

// admin entrypoint to update SC code for any update
Function SCUpdate(code String) Uint64
1 IF LOAD("owner") != SIGNER() THEN GOTO 4
2 UPDATE_SC_CODE(code)
3 RETURN 0
4 RETURN 1
End Function