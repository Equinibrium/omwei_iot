# Ambient temperature

Registry descriptor: `0x0042`

Meaning:
- ambient temperature
- canonical unit: degree Celsius
- encoding: signed int32
- scale: 0.001

For a measured value of 22.500 °C:

`22500`

Minimal representation:

`0x0042 | 22500`

The registry supplies the semantic label, unit, datatype, and scale; these do not have to be repeated in every observation.
