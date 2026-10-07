# 21.07 — Black & White and Channel Mixer Adjustments

# Black & White

ID [ptnd.adjustment.black](http://ptnd.adjustment.black)_white.

Converts color to grayscale appearance using adjustable source color contributions (red/yellow/green/cyan/blue/magenta or model-defined bands). Output remains in document rendering context unless explicit grayscale conversion command.

# Tint

Optional tint ColorValue + strength, implemented after luminance mix.

# Channel Mixer

ID [ptnd.adjustment.channel](http://ptnd.adjustment.channel)_mixer.

Output channel is weighted combination of input channels plus constant. Matrix/offset representation is canonical.

# Monochrome mode

Channel mixer can produce monochrome using one output formula; UI labels mode.

# Alpha

Unaffected unless alpha channel mixer is separately supported.

# Tests

Identity matrices, grayscale patches, negative coefficients, sum >100%, tint, masks and serialization.