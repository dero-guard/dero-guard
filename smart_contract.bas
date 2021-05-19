//https://github.com/Slixe/dero-guard
Function Initialize() Uint64
10 STORE("total", 0)
11 RETURN 0
End Function

// Use to register as an exit node provider.
Function Register(price Uint64, country String) Uint64
10 STORE(SIGNER() + "_price", price)
11 STORE(SIGNER() + "_country", country)
20 STORE("total", LOAD("total") + 1)
30 RETURN 0
End Function

// Allows anyone to rate a provider.
Function Note(value Uint64, provider String) Uint64
10 IF value > 0 AND value < 5 THEN GOTO 12
11 RETURN 1
12 IF IS_ADDRESS_VALID(provider) AND EXISTS(provider + "_price") THEN GOTO 15
13 RETURN 1
15 STORE(provider + "_trust_" + SIGNER(), value)
30 RETURN 0
End Function