# 03.7 — Photo Color, Histogram, Channels, Scopes, Soft Proof & Sampling

# Histogram

Asynchronous per-channel/luminance analysis from selected scope. Shows clipping and refresh status. Does not block canvas.

# Channels

Composite + color channels + alpha/spot where semantic. Visibility is view state; active edit channel is explicit target.

# Picker

Point/average radius; Current Layer/Composite; document numeric values and display values distinguished.

# Info

Coordinates, color components, profile/model, alpha and sample statistics.

# Soft proof

Uses shared color engine; Photo can compare document appearance under target profile. Gamut warning overlay is view-only.

# Curves/Levels backdrop

Histogram data integrated as derived visualization. Curve controls offer numeric/accessibility alternative.

# Scopes roadmap

Waveform, vectorscope and advanced analysis can implement AnalysisProvider interface post-V1.

# Performance

Analysis jobs coalesce on revision; moving a slider should not enqueue unbounded full-resolution histograms.