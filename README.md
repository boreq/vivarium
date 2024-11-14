# Vivarium

Automation and designs for my vivarium. This readme was written as a desperate
attempt at helping future me when something inevitably breaks at the worst
possible moment.

## Notes

### Plants

- [*Pellaea rotundifolia*](https://duckduckgo.com/?q=Pellaea+rotundifolia&iar=images&iax=images&ia=images)
- [*Hemionitis arifolia*](https://duckduckgo.com/?q=Hemionitis+arifolia&iax=images&ia=images)
- [*Nephrolepis cordifolia ‘Duffii’ Nefrolepis*](https://duckduckgo.com/?q=Nephrolepis+cordifolia+%E2%80%98Duffii%E2%80%99+Nefrolepis&iar=images&iax=images&ia=images)
- [*Pyrrosia nummularifolia*](https://duckduckgo.com/?q=Pyrrosia+nummularifolia&iax=images&ia=images)
- [*Phymatosorus diversifolius*](https://duckduckgo.com/?q=Phymatosorus+diversifolius&iar=images&iax=images&ia=images)
- [*Microgramma nitida*](https://duckduckgo.com/?q=Microgramma+nitida&iar=images&iax=images&ia=images)
- [*Pleurozium schreberi*](https://duckduckgo.com/?q=Pleurozium+schreberi&iax=images&ia=images)
- [*Polytrichum commune*](https://duckduckgo.com/?q=Polytrichum+commune&iax=images&ia=images)
- [*Microsorium pteropus var petite*](https://duckduckgo.com/?q=Microsorium+pteropus+var+petite&iar=images&iax=images&ia=images)
- [*Limnobium laevigatum*](https://duckduckgo.com/?q=Limnobium+laevigatum&iar=images&iax=images&ia=images)

### Planting

There is a layer of sphagnum moss + orchid bark + coco coir mixed 1-1-1 at the
bottom of the tank. There is only spaghnum moss in the pots.

### Automation 

See `./automation`, use `kicad`. The schematic doesn't have pulldowns on pins as
it seems to work without them because of reasons.

GPIO pins used are `{22, 23, 24, 25}` as per schematic:
- `GPIO 22` = relay `M4` = output `?`
- `GPIO 23` = relay `M3` = output `?`
- `GPIO 24` = relay `M2` = output `?`
- `GPIO 25` = relay `M1` = output `?`


This is `Raspberry Pi 1 Model B`.

Raspberry Pi OS is
[here](https://www.raspberrypi.com/software/operating-systems/), use lite. It's
debian. Arch Linux ARM doesn't support this Pi out of the box anymore.

The wifi dongle is `TP-Link TL-WN725N` and was chosen without any thought put into it.

The config is in `/etc/network/interfaces`. The manual is
[here](https://wiki.debian.org/NetworkConfiguration). The manual for WIFI is
[here](https://wiki.debian.org/WiFi/HowToUse#Manual).

Wired connection config would look something like this:

```
auto eth0
iface eth0 inet static
	address 192.168.0.30/24
	gateway 192.168.0.1/24
```

WIFI config would look something like this:

```
auto wlan0
allow-hotplug wlan0
iface wlan0 inet dhcp
        wpa-ssid somessid
        wpa-psk somepassword
```

The login info is on the device. SSH isn't running on port 22.

### Terrarium

[RNT-45 from Aqua Nova/Reptile Nova](http://archive.today/2024.11.03-024309/https://www.aqua-nova.pl/?a=produkty&opcja=show&idprod=1800&idkat=55).

### Lid

The material is 5mm thick foamed PVC.

The glue is silicone branded "for aquarium glass" as that's what I had lying
around.

Open `lid.dxf` with `librecad` in case you need to redo a panel.

### Light

Those are the parameters for the LED strip supposedly but who knows:

```
Voltage: 12V DC
Color: 4000-4500K
LEDs per meter: 320
Flux: 900lm/m
Recommended power supply: 10W per meter
```

The strips were self-adhesive, no extra glue was used.

The power connector is one of the standard DC connectors, `2.5/5.5` or
`2.1/5.5`. The connector on this power supply was replaced and it unscrews. It
can be reused.

### False bottom

Use 20 PPI foam, ordering 45 PPI was a fuck up.

### Internal dividers

Don't use the metal mesh for water inlets, its so annoying to work with. Next
time the pump compartment can be slightly smaller I think.

### Waterfall

The pump hose is a 10mm one. 

The waterfall was spraying at the back wall and had to be redone as the water
was leaking. Checking for leaks as early as possible was a really good idea.