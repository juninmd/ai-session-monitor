// OpenCode mark in one ink: the frame is solid, the inner block is the same ink at 30%.
// Path data is vendor/opencode.mono.svg, untouched; the viewBox is cropped around the glyph.
import { type LogoProps, LogoSvg } from './LogoSvg';

const BLOCK = 'M320 224V352H192V224H320Z';
const FRAME = 'M384 416H128V96H384V416ZM320 160H192V352H320V160Z';

export function OpenCodeLogo(props: LogoProps) {
  return (
    <LogoSvg harness="opencode" tone="mono" viewBox="64 64 384 384" {...props}>
      <path d={BLOCK} fill="currentColor" fillOpacity="0.3" />
      <path d={FRAME} fill="currentColor" fillRule="evenodd" clipRule="evenodd" />
    </LogoSvg>
  );
}
