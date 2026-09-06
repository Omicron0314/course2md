// Run from the repository root: swift desktop/assets/dmg/render.swift
// Native SF typography, with 1x and 2x representations in one Finder background.
import AppKit

let output = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let size = NSSize(width: 760, height: 540)

func color(_ hex: UInt32) -> NSColor {
    NSColor(srgbRed: CGFloat((hex >> 16) & 255) / 255,
            green: CGFloat((hex >> 8) & 255) / 255,
            blue: CGFloat(hex & 255) / 255, alpha: 1)
}

func text(_ value: String, y: CGFloat, size: CGFloat, weight: NSFont.Weight, color: NSColor) {
    let style = NSMutableParagraphStyle()
    style.alignment = .center
    (value as NSString).draw(in: NSRect(x: 40, y: y, width: 680, height: size * 1.7), withAttributes: [
        .font: NSFont.systemFont(ofSize: size, weight: weight),
        .foregroundColor: color,
        .paragraphStyle: style,
    ])
}

var representations: [NSBitmapImageRep] = []
for scale in [1, 2] {
    let bitmap = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: 760 * scale,
                                 pixelsHigh: 540 * scale, bitsPerSample: 8, samplesPerPixel: 4,
                                 hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB,
                                 bytesPerRow: 0, bitsPerPixel: 0)!
    let context = NSGraphicsContext(bitmapImageRep: bitmap)!
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = context
    context.cgContext.scaleBy(x: CGFloat(scale), y: CGFloat(scale))
    context.cgContext.translateBy(x: 0, y: size.height)
    context.cgContext.scaleBy(x: 1, y: -1)
    NSGraphicsContext.current = NSGraphicsContext(cgContext: context.cgContext, flipped: true)

    color(0xF6F7F9).setFill()
    NSRect(origin: .zero, size: size).fill()
    text("course2md", y: 46, size: 32, weight: .semibold, color: color(0x202936))
    text("Video to Markdown", y: 91, size: 15, weight: .regular, color: color(0x798391))

    let arrow = NSBezierPath()
    arrow.move(to: NSPoint(x: 344, y: 224))
    arrow.line(to: NSPoint(x: 414, y: 224))
    arrow.move(to: NSPoint(x: 398, y: 208))
    arrow.line(to: NSPoint(x: 414, y: 224))
    arrow.line(to: NSPoint(x: 398, y: 240))
    arrow.lineWidth = 4
    arrow.lineCapStyle = .round
    arrow.lineJoinStyle = .round
    color(0x8D9BAD).setStroke()
    arrow.stroke()

    text("拖到右侧「应用程序」完成安装", y: 358, size: 16, weight: .medium, color: color(0x445266))
    text("Drag course2md to Applications to install.", y: 387, size: 13, weight: .regular, color: color(0x7B8593))
    NSGraphicsContext.restoreGraphicsState()
    bitmap.size = size
    representations.append(bitmap)
    if scale == 2 {
        try bitmap.representation(using: .png, properties: [:])!.write(to: output.appendingPathComponent("background@2x.png"))
    }
}
let image = NSImage(size: size)
image.addRepresentations(representations)
try image.tiffRepresentation(using: .lzw, factor: 0)!.write(to: output.appendingPathComponent("background.tiff"))
print("Rendered 760 × 540 pt Finder background at 1x and 2x.")
