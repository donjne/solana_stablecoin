# Stable.fun

```bash
src/
├── lib.rs                 # Program 
├── state/                # Program state
│   ├── mod.rs
│   └── factory.rs        # Global factory
│   └── stablecoin.rs     
│   └── others ...        
├── contexts/             # Program contexts
│   ├── mod.rs
│   ├── init_factory.rs
│   ├── init_stablecoin.rs
│   └── mint_stablecoin.rs
├── errors.rs            # Errors
├── events.rs            # Events
└── constants.rs         # Program constants
```
