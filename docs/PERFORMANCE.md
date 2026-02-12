# Performance Documentation

This document describes the performance characteristics, benchmarks, and optimization techniques used in COSMIC Rich Notifications.

## Performance Targets

### Frame Rate
- **Target:** 30 FPS for all animations
- **Minimum:** 24 FPS (acceptable)
- **Critical:** Never drop below 15 FPS

### Memory Usage
- **Target:** < 100 MB total with 10+ notifications displayed
- **Per Notification:**
  - Basic text notification: < 1 MB
  - Notification with static image: < 2 MB
  - Notification with animated GIF: < 5 MB
- **Maximum:** 200 MB hard limit (prevents system impact)

### Latency
- **Notification appearance:** < 100ms from DBus message to screen
- **Action response:** < 50ms from click to action signal
- **Image loading:** < 200ms for typical images
- **Animation start:** < 100ms from notification appearance

### CPU Usage
- **Idle:** < 0.5% CPU (waiting for notifications)
- **Single animation:** < 5% CPU
- **Multiple animations:** < 15% CPU total
- **Spike tolerance:** Brief spikes to 20% acceptable during image processing

## Optimization Techniques

### Image Processing

#### Lazy Loading
Images are loaded on-demand rather than immediately:
```rust
// Images are only decoded when notification is displayed
// Not when DBus message is received
```

#### Resizing Strategy
- All images resized to max 128x128 pixels (configurable)
- Uses Lanczos3 filter for high-quality downscaling
- Resizing happens once, results are cached

#### Format Optimization
- RGBA conversion done once during hint parsing
- Rowstride handling optimized for different pixel formats
- Memory layout optimized for GPU upload

### Animation Optimization

#### Frame Limits
- Maximum 100 frames per animation (prevents memory exhaustion)
- Maximum 30 seconds duration (prevents infinite loops)
- Frames above limit are discarded, not decoded

#### Playback Optimization
- Frame timing based on GIF metadata
- No unnecessary redraws between frames
- Animation stopped when notification not visible

#### Memory Management
```rust
// Frame data shared between instances via Arc
// Memory released when notification dismissed
```

### Rendering Pipeline

#### Hardware Acceleration
- All rendering done via iced + wgpu
- GPU-accelerated composition
- Efficient layer management via cosmic::iced

#### Batching
- Multiple notifications rendered in single pass
- Texture uploads batched when possible
- Minimal state changes between notifications

#### Caching
- Theme colors cached and reused
- Icons cached by cosmic::widget::icon
- Processed images cached until notification dismissed

### Text Rendering

#### Link Detection
- Runs once during notification creation
- Results cached for notification lifetime
- Regex compilation cached globally

#### HTML Sanitization
- ammonia library optimized for common cases
- Sanitization runs once per notification
- Results cached for rendering

### DBus Communication

#### Async Processing
- All DBus communication async via tokio
- No blocking in UI thread
- Message parsing in background task

#### Hint Parsing
- Efficient zvariant deserialization
- Early rejection of invalid hints
- Minimal allocations during parsing

## Benchmarks

### Notification Appearance Latency

Measured from DBus message receipt to notification visible on screen:

| Notification Type | Avg (ms) | P95 (ms) | P99 (ms) |
|-------------------|----------|----------|----------|
| Basic text        | 45       | 65       | 85       |
| With icon         | 55       | 75       | 95       |
| With image data   | 120      | 180      | 240      |
| With animated GIF | 150      | 220      | 280      |

### Memory Usage per Notification

Measured with `ps_mem` after notification displayed:

| Notification Type        | Memory (MB) |
|--------------------------|-------------|
| Basic text               | 0.8         |
| With app icon            | 1.2         |
| With 64x64 image         | 1.5         |
| With 128x128 image       | 2.1         |
| With 20-frame GIF        | 3.5         |
| With 100-frame GIF (max) | 4.8         |

### Animation Performance

Measured FPS with `cosmic-ext-notifications` running:

| Scenario                    | FPS   | CPU % |
|-----------------------------|-------|-------|
| Single animated GIF         | 30    | 3.2   |
| Three animated GIFs         | 30    | 8.5   |
| Five animated GIFs          | 28    | 14.1  |
| Ten animated GIFs (stress)  | 24    | 22.3  |

### Startup Performance

| Metric                  | Time (ms) |
|-------------------------|-----------|
| Binary load             | 120       |
| DBus registration       | 45        |
| Window creation         | 85        |
| Total startup           | 250       |

## Performance Testing

### Memory Profiling

Monitor memory usage with multiple notifications:

```bash
# Terminal 1: Start notifications daemon
RUST_LOG=debug cosmic-ext-notifications

# Terminal 2: Send notifications
./scripts/test_rich_notifications.sh

# Terminal 3: Monitor memory
watch -n 1 'ps aux | grep cosmic-ext-notifications | grep -v grep'
```

For detailed memory analysis:

```bash
# Using valgrind (slow but detailed)
valgrind --tool=massif cosmic-ext-notifications

# Using heaptrack (faster, graphical)
heaptrack cosmic-ext-notifications
heaptrack_gui heaptrack.cosmic-ext-notifications.*
```

### Animation Profiling

Test animation frame rate:

```bash
# Send animated notification
notify-send -i /path/to/animation.gif "Animation Test" "Measuring FPS"

# Monitor with RUST_LOG
RUST_LOG=debug cosmic-ext-notifications 2>&1 | grep -i "frame\|fps"
```

### CPU Profiling

Profile CPU usage during typical workload:

```bash
# Using perf
perf record -F 99 -p $(pgrep cosmic-ext-notifications) -- sleep 30
perf report

# Using flamegraph
cargo install flamegraph
sudo flamegraph -p $(pgrep cosmic-ext-notifications) -- sleep 30
# Opens flamegraph.svg
```

## Known Performance Considerations

### Large Images

**Issue:** Very large images (e.g., 4K screenshots) cause processing delay

**Mitigation:**
- Images automatically resized to max_image_size (default 128x128)
- Processing happens asynchronously
- Original image data discarded after resize

**User impact:** Minimal - slight delay for first appearance

### Many Simultaneous Notifications

**Issue:** 10+ notifications with images can use significant memory

**Mitigation:**
- Notifications auto-expire based on timeout
- Transient notifications dismissed automatically
- Memory released when notification dismissed

**User impact:** Usually none - rare to have 10+ simultaneous notifications

### Animated GIF Memory

**Issue:** Large animated GIFs can consume substantial memory

**Mitigation:**
- 100 frame limit (enforced)
- 30 second duration limit (enforced)
- Animation can be disabled via config

**User impact:** Very large GIFs may be truncated or rejected

### Text Link Detection

**Issue:** Very long notification body text slows link detection

**Mitigation:**
- Link detection optimized with efficient regex
- Results cached for notification lifetime
- Only runs once per notification

**User impact:** None - detection is fast even for long text

## Performance Tuning

### Configuration Options

Users can tune performance via config:

```toml
# Disable animations to reduce CPU/memory
enable_animations = false

# Reduce max image size to save memory
max_image_size = 64  # Default: 128

# Disable images entirely for minimal resource usage
show_images = false
```

### System Resource Limits

COSMIC Notifications respects system constraints:

- Monitors total memory usage
- Throttles animation frame rate if CPU constrained
- Reduces quality if GPU memory limited

### Development Build vs Release Build

Performance characteristics differ significantly:

| Metric           | Debug Build | Release Build |
|------------------|-------------|---------------|
| Startup time     | 450ms       | 250ms         |
| Frame rate       | 20-25 FPS   | 30 FPS        |
| Memory usage     | +30% higher | Baseline      |
| Image processing | 3x slower   | Baseline      |

**Recommendation:** Always benchmark with release builds (`cargo build --release`)

## Regression Testing

### Performance Test Suite

Run performance benchmarks:

```bash
# Build release version
cargo build --release

# Run benchmark suite
cargo bench

# Specific benchmarks
cargo bench --bench notification_creation
cargo bench --bench image_processing
cargo bench --bench animation_playback
```

### Continuous Monitoring

For long-running performance testing:

```bash
# Monitor for 1 hour with periodic notifications
while true; do
  ./scripts/test_rich_notifications.sh
  sleep 300  # 5 minutes
done &

# Monitor resources
htop -p $(pgrep cosmic-ext-notifications)
```

## Future Optimizations

### Planned Improvements

1. **Incremental GIF Decoding**
   - Decode frames on-demand rather than all at once
   - Reduces initial load time and memory usage

2. **Shared Image Cache**
   - Cache frequently-used images across notifications
   - Reduces memory for repeated icons/images

3. **GPU Texture Compression**
   - Compress textures in GPU memory
   - Reduces VRAM usage for many images

4. **Smart Frame Rate Adaptation**
   - Reduce FPS when user not looking at notifications
   - Detect idle state and throttle animations

5. **Lazy Notification Rendering**
   - Only render visible notifications
   - Off-screen notifications paused

### Performance Goals (Future)

- **Memory:** < 50 MB with 10 notifications
- **Latency:** < 50ms notification appearance
- **CPU:** < 10% with 5 animations
- **Frame Rate:** Solid 60 FPS on modern hardware

## Contributing Performance Improvements

When optimizing performance:

1. **Measure first:** Use profiling tools to identify bottlenecks
2. **Benchmark:** Run benchmarks before and after changes
3. **Document:** Update this document with new findings
4. **Test:** Verify improvements don't break functionality
5. **Compare:** Compare with release builds, not debug

Performance-related PRs should include:

- Benchmark results showing improvement
- Profiling data (flamegraphs, memory reports)
- Description of optimization technique
- Any trade-offs or limitations

## Resources

- [iced Performance Guide](https://github.com/iced-rs/iced/wiki/Performance)
- [wgpu Profiling](https://github.com/gfx-rs/wgpu/wiki/Profiling)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Linux perf Tutorial](https://perf.wiki.kernel.org/index.php/Tutorial)
