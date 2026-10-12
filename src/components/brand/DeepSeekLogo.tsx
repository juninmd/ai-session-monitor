// A plain "D" drawn for this project in one ink. It is a neutral glyph, not DeepSeek's artwork:
// original/deepseek.mono.svg is the source file and docs/third-party-assets.md says why.
import { type LogoProps, LogoSvg } from './LogoSvg';

const GLYPH =
  'M5 3H12.5C17.5 3 21 6.8 21 12S17.5 21 12.5 21H5ZM8.4 6.3V17.7H12.4C15.5 17.7 17.6 15.4 17.6 12S15.5 6.3 12.4 6.3Z';

export function DeepSeekLogo(props: LogoProps) {
  return (
    <LogoSvg harness="deepseek" tone="mono" viewBox="0 0 24 24" {...props}>
      <path d={GLYPH} fill="currentColor" fillRule="evenodd" />
    </LogoSvg>
  );
}
