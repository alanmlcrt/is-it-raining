import { Geist_400Regular, Geist_500Medium } from '@expo-google-fonts/geist';
import { useFonts } from 'expo-font';
import { StatusBar } from 'expo-status-bar';
import { useCallback, useEffect, useRef, useState } from 'react';
import {
  AccessibilityInfo,
  Animated,
  AppState,
  Easing,
  Platform,
  RefreshControl,
  ScrollView,
  StatusBar as NativeStatusBar,
  StyleSheet,
  Text,
  View,
} from 'react-native';

import {
  answer,
  bandSentence,
  barHeight,
  clock,
  type Rain,
  sourceLine,
  spokenAnswer,
  type Step,
  upcoming,
} from './src/rain';
import { band, motion, space, type, useColors } from './src/theme';

const API_URL = process.env.EXPO_PUBLIC_API_URL;
// ponytail: lieu fixe, dans la zone du serveur (D9). Position et recherche de ville : B3.
const PLACE = { name: 'Paris', lat: 48.86, lon: 2.35 };
const REFRESH_MS = 60_000;
const TIMEOUT_MS = 10_000;
// ponytail: marges système approchées. react-native-safe-area-context ne compile pas sous
// Windows (chemins trop longs pour ninja). À remettre avec HeroUI (B4), qui l'exige, et
// indispensable pour iOS : encoche et barre d'accueil ne sont pas gérées ici.
const INSET_TOP = Platform.OS === 'android' ? (NativeStatusBar.currentHeight ?? 0) : 48;
const INSET_BOTTOM = 48;

/** La dernière réponse, ou `null` dès qu'un appel échoue : jamais de réponse ancienne à l'écran. */
function useRain() {
  const [rain, setRain] = useState<Rain | null>(null);
  const [failed, setFailed] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [now, setNow] = useState(() => new Date());

  const load = useCallback(async () => {
    const abort = new AbortController();
    const timer = setTimeout(() => abort.abort(), TIMEOUT_MS);
    try {
      const url = `${API_URL}/v1/rain?lat=${PLACE.lat}&lon=${PLACE.lon}`;
      const res = await fetch(url, { signal: abort.signal });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      setRain(await res.json());
      setFailed(false);
    } catch {
      setRain(null);
      setFailed(true);
    } finally {
      clearTimeout(timer);
      setNow(new Date());
    }
  }, []);

  const refresh = useCallback(async () => {
    setRefreshing(true);
    await load();
    setRefreshing(false);
  }, [load]);

  useEffect(() => {
    load();
    const interval = setInterval(load, REFRESH_MS);
    const appState = AppState.addEventListener('change', (state) => {
      if (state === 'active') load();
    });
    return () => {
      clearInterval(interval);
      appState.remove();
    };
  }, [load]);

  return { rain, failed, refreshing, refresh, now };
}

function RainBand({ steps, rain }: { steps: Step[]; rain: Rain | null }) {
  const colors = useColors();
  const rise = useRef(new Animated.Value(0)).current;
  const wet = rain?.state === 'rain_soon' || rain?.state === 'raining';

  useEffect(() => {
    if (!wet) return;
    AccessibilityInfo.isReduceMotionEnabled().then((reduced) => {
      if (reduced) {
        rise.setValue(1);
        return;
      }
      Animated.timing(rise, {
        toValue: 1,
        duration: motion.barsMs,
        easing: Easing.bezier(...motion.curve),
        useNativeDriver: true,
      }).start();
    });
  }, [wet, rise]);

  if (steps.length === 0) return null;
  const middle = steps[Math.floor(steps.length / 2)];
  const last = steps[steps.length - 1];

  return (
    <View accessible accessibilityLabel={bandSentence(rain)}>
      <Animated.View
        style={[styles.bars, { transform: [{ scaleY: rise }], transformOrigin: 'bottom' }]}
      >
        {wet &&
          steps.map((step) => (
            <View key={step.t} style={styles.barSlot}>
              <View
                style={{
                  height: barHeight(step.mm_h) * band.height,
                  backgroundColor: colors.rain,
                }}
              />
            </View>
          ))}
      </Animated.View>
      <View style={[styles.baseline, { backgroundColor: colors.hairline }]} />
      <View style={styles.hours}>
        {[steps[0], middle, last].map((step) => (
          <Text key={step.t} style={[type.caption, styles.tabular, { color: colors.text3 }]}>
            {clock(step.t)}
          </Text>
        ))}
      </View>
    </View>
  );
}

function Screen() {
  const colors = useColors();
  const { rain, failed, refreshing, refresh, now } = useRain();
  const reply = answer(rain, now);
  const known = reply.big !== undefined;

  return (
    <ScrollView
      style={{ backgroundColor: colors.bg }}
      contentContainerStyle={[
        styles.page,
        { paddingTop: INSET_TOP + space.m, paddingBottom: INSET_BOTTOM + space.m },
      ]}
      refreshControl={
        <RefreshControl refreshing={refreshing} onRefresh={refresh} tintColor={colors.text2} />
      }
    >
      <Text accessibilityRole="header" style={[type.detail, { color: colors.text }]}>
        {PLACE.name}
      </Text>

      <View accessible accessibilityLabel={spokenAnswer(reply)} style={styles.answer}>
        <Text style={[type.lead, { color: known ? colors.text : colors.text2 }]}>{reply.lead}</Text>
        {known && (
          <Text
            adjustsFontSizeToFit
            numberOfLines={1}
            style={[type.answer, styles.tabular, { color: colors.text }]}
          >
            {reply.big}
          </Text>
        )}
        {reply.detail && (
          <Text style={[type.detail, { color: colors.text2 }]}>{reply.detail}</Text>
        )}
        {failed && <Text style={[type.detail, { color: colors.text2 }]}>Serveur injoignable</Text>}
      </View>

      <RainBand steps={upcoming(rain, now)} rain={rain} />

      <Text style={[type.caption, styles.source, { color: colors.text3 }]}>
        {sourceLine(rain, now)}
      </Text>
      <StatusBar style="auto" />
    </ScrollView>
  );
}

export default function App() {
  const [fontsLoaded] = useFonts({ Geist_400Regular, Geist_500Medium });
  if (!fontsLoaded) return null;
  return <Screen />;
}

const styles = StyleSheet.create({
  page: { flexGrow: 1, paddingHorizontal: space.edge },
  answer: { flex: 1, justifyContent: 'center', paddingVertical: space.block },
  bars: { flexDirection: 'row', alignItems: 'flex-end', height: band.height, gap: space.bar },
  barSlot: { flex: 1 },
  baseline: { height: StyleSheet.hairlineWidth },
  hours: { flexDirection: 'row', justifyContent: 'space-between', marginTop: space.s },
  source: { marginTop: space.m },
  tabular: { fontVariant: ['tabular-nums'] },
});
