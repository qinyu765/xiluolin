import {
  BookOpen,
  Bot,
  Briefcase,
  ClipboardList,
  Code,
  Flame,
  Heart,
  Languages,
  Lightbulb,
  Mail,
  Palette,
  Sparkles,
  Star,
  Target,
  Zap,
  type LucideIcon,
} from "lucide-react";

export const PERSONA_ICONS = {
  Bot,
  ClipboardList,
  Lightbulb,
  Mail,
  Languages,
  Sparkles,
  Zap,
  Target,
  Palette,
  BookOpen,
  Code,
  Briefcase,
  Heart,
  Star,
  Flame,
} satisfies Record<string, LucideIcon>;

export type PersonaIconName = keyof typeof PERSONA_ICONS;

export const PERSONA_ICON_NAMES = Object.keys(
  PERSONA_ICONS,
) as PersonaIconName[];

export function getPersonaIcon(iconName: string): LucideIcon | undefined {
  return PERSONA_ICONS[iconName as PersonaIconName];
}
