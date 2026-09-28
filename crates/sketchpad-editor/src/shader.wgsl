struct VOut {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
};
@vertex fn vs(@location(0) position: vec2f, @location(1) color: vec4f) -> VOut {
    var out: VOut;
    out.position = vec4f(position, 0.0, 1.0);
    out.color = color;
    return out;
}
@fragment fn fs(input: VOut) -> @location(0) vec4f {
    return input.color;
}
