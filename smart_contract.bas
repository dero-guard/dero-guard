//https://github.com/Slixe/dero-guard
Function Initialize() Uint64
10 STORE("total", 0)
11 RETURN 0
End Function

// Use to register as an exit node provider.
Function Register(price Uint64, name String, country String, addr String) Uint64
//1 IF !IS_ADDRESS_VALID(addr) THEN GOTO 30 <-- Not working 
2 DIM exist As Uint64
3 LET exist = EXISTS(addr)
4 DIM current As Uint64
5 IF exist THEN GOTO 8
6 LET current = LOAD("total")
7 GOTO 10
8 LET current = LOAD(addr)
10 STORE("provider_" + current + "_price", price)
11 STORE("provider_" + current + "_name", name)
12 STORE("provider_" + current + "_country", country)
13 STORE("provider_" + current + "_address", addr)
14 STORE(addr, 1)
15 IF exist THEN GOTO 30
20 STORE("total", LOAD("total") + 1)
30 RETURN 0
End Function

// Allows anyone to rate a provider.
Function Note(value Uint64, provider Uint64) Uint64
10 IF value > 0 AND value < 5 THEN GOTO 12
11 RETURN 1
12 IF EXISTS(provider + "_price") THEN GOTO 15
13 RETURN 1
15 STORE(provider + "_trust_" + SIGNER(), value)
30 RETURN 0
End Function