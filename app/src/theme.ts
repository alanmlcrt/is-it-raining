import { useColorScheme } from 'react-native';

// Tokens de docs/04-design.md. Aucune couleur ni espacement brut ailleurs.
// ponytail: thème en TypeScript seul, à déclarer aussi dans HeroUI quand les réglages arrivent (B4).

const dark = {
  bg: '#141415',
  text: '#EDEDED',
  text2: '#A1A1A6',
  text3: '#85858B',
  hairline: 'rgba(237, 237, 237, 0.08)',
  rain: '#F5C518',
};

const light: typeof dark = {
  bg: '#FAFAFA',
  text: '#171717',
  text2: '#525252',
  text3: '#737373',
  hairline: 'rgba(23, 23, 23, 0.08)',
  rain: '#B38900',
};

export const space = { bar: 2, s: 8, m: 16, edge: 24, block: 32 };

export const band = { height: 96 };

export const font = { regular: 'Geist_400Regular', medium: 'Geist_500Medium' };

export const type = {
  answer: { fontFamily: font.medium, fontSize: 88, lineHeight: 92 },
  lead: { fontFamily: font.regular, fontSize: 28, lineHeight: 34 },
  detail: { fontFamily: font.regular, fontSize: 17, lineHeight: 24 },
  caption: { fontFamily: font.regular, fontSize: 13, lineHeight: 20 },
};

export const motion = { barsMs: 300, curve: [0.22, 1, 0.36, 1] as const };

export function useColors() {
  return useColorScheme() === 'dark' ? dark : light;
}
