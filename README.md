# Vivarium

Automation and designs for my vivarium. This readme was written as a desperate
attempt at helping future me when something inevitably breaks at the worst
possible moment.

## Vivarium

### Plants

- [*Pellaea rotundifolia*](https://duckduckgo.com/?q=Pellaea+rotundifolia&iar=images&iax=images&ia=images)
- [*Hemionitis arifolia*](https://duckduckgo.com/?q=Hemionitis+arifolia&iax=images&ia=images)
- [*Nephrolepis cordifolia* ‘Duffii’](https://duckduckgo.com/?q=Nephrolepis+cordifolia+%E2%80%98Duffii%E2%80%99+Nefrolepis&iar=images&iax=images&ia=images)
- [*Pyrrosia nummularifolia*](https://duckduckgo.com/?q=Pyrrosia+nummularifolia&iax=images&ia=images)
- [*Phymatosorus diversifolius*](https://duckduckgo.com/?q=Phymatosorus+diversifolius&iar=images&iax=images&ia=images)
- [*Microgramma nitida*](https://duckduckgo.com/?q=Microgramma+nitida&iar=images&iax=images&ia=images)
- [*Pleurozium schreberi*](https://duckduckgo.com/?q=Pleurozium+schreberi&iax=images&ia=images)
- [*Polytrichum commune*](https://duckduckgo.com/?q=Polytrichum+commune&iax=images&ia=images)
- [*Microsorium pteropus* var. *petite*](https://duckduckgo.com/?q=Microsorium+pteropus+var+petite&iar=images&iax=images&ia=images)
- [*Limnobium laevigatum*](https://duckduckgo.com/?q=Limnobium+laevigatum&iar=images&iax=images&ia=images)
- [*Anubias barteri* var. *nana*](https://duckduckgo.com/?t=ffab&q=Anubias+barteri+var.+nana&iax=images&ia=images)

### Animals

- [*Armadillidium klugii*](https://duckduckgo.com/?hps=1&q=Armadillidium+klugii&iax=images&ia=images)
- [*Caridina multidentata*](https://duckduckgo.com/?hps=1&q=Caridina+multidentata&iax=images&ia=images)
- [*Clithon corona*](https://duckduckgo.com/?hps=1&q=Clithon+corona&iax=images&ia=images)
- [Unspecified species of springtails](https://duckduckgo.com/?q=vivarium++springtails&t=ffab&iar=images&iax=images&ia=images)
- [*Lepidodactylus lugubris*](https://duckduckgo.com/?t=ffab&q=Lepidodactylus+lugubris&iax=images&ia=images)

### DICP (Dynamic Isopod Configuration Protocol) identifiers

- Baltazar
- Brunhilda

### Planting

There is a layer of sphagnum moss + orchid bark + coco coir mixed 1-1-1 at the
bottom of the tank. There is only spaghnum moss in the pots.

### Automation 

See `./automation`, use `kicad`. The schematic doesn't have pulldowns on pins as
it seems to work without them because of reasons.

GPIO pins used for relays are:
- `GPIO 22` = output `3`
- `GPIO 23` = output `4`
- `GPIO 24` = output `2`
- `GPIO 25` = output `1`

GPIO pins used for the water tank sensor are:
- `GPIO 17` = trig
- `GPIO 18` = echo

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

The strips were self-adhesive, no extra glue was used which was a mistake as
they started coming off, perhaps due to moisture. I'm using the strong double
sided tape to put them back on. We will see if that will help.

The power connector is one of the standard DC connectors, `2.5/5.5` or
`2.1/5.5`. The connector on this power supply was replaced and it unscrews. It
can be reused. The strips are 12V.

#### Strip 1

Those are the parameters for the LED strip supposedly but who knows:

```
Color: 4000-4500K
LEDs per meter: 320
Flux: 900lm/m
Recommended power supply: 10W per meter (lol)
```

Basically the whole thing was used, I just measured the amps to be around 0.85A
which is closer to 2W per meter.

#### Strip 2

```
Color: 4000K
LEDs per meter: 320
```

#### UV

[Repti Planet UVB 5.0
Tropical](https://web.archive.org/web/20250115155401/https://reptiplanet.pet/portfolio-items/repti-planet-repti-uvb-5-0-3/),
running since 2025-01-15.

### False bottom

Use 20 PPI foam, ordering 45 PPI was a fuck up.

### Internal dividers

Don't use the metal mesh for water inlets, its so annoying to work with. Next
time the pump compartment can be slightly smaller I think.

### Waterfall

The pump hose is a 10mm one. 

The waterfall was spraying at the back wall and had to be redone as the water
was leaking. Checking for leaks as early as possible was a really good idea.

## Isopodarium

### Animals

- [*Porcellio scaber* "Dalmatian"](https://duckduckgo.com/?q=Porcellio+scaber+%22Dalmatian%22&iar=images)
- [*Cubaris* sp. "Panda King"](https://duckduckgo.com/?q=Cubaris+sp.+"Panda+King"&iar=images)
