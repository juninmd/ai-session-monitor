import { paceLine, type ViewContext, type WindowModel } from '../../app/windowModel';
import { Icon } from '../icons';

interface PaceLineProps {
  model: WindowModel;
  ctx: ViewContext;
  /** Compact rows only show the urgent lines (out, at risk). */
  onlyUrgent?: boolean;
  className?: string;
}

export function PaceLine({ model, ctx, onlyUrgent = false, className = 'pace' }: PaceLineProps) {
  const line = paceLine(model, ctx);
  if (line === null || (onlyUrgent && !line.urgent)) return null;
  return (
    <p className={className}>
      <Icon name={line.icon} />
      <span>{line.text}</span>
    </p>
  );
}
