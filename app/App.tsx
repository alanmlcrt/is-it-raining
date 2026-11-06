import { Geist_400Regular, Geist_500Medium } from '@expo-google-fonts/geist';
import { useFonts } from 'expo-font';
import * as Location from 'expo-location';
import { StatusBar } from 'expo-status-bar';
import { useCallback, useEffect, useRef, useState } from 'react';
import {
  AccessibilityInfo,
  Animated,
  AppState,
  Easing,
  Linking,
  Platform,
  Pressable,
  RefreshControl,
  ScrollView,
  StatusBar as NativeStatusBar,
  StyleSheet,
  Text,
  View,
} from 'react-native';

import { type Place, round2 } from './src/place';
import { PlaceSheet } from './src/PlaceSheet';
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
import { band, motion, size, space, type, useColors } from './src/theme';

const API_URL = process.env.EXPO_PUBLIC_API_URL;
const REFRESH_MS = 60_000;
const TIMEOUT_MS = 10_000;
/** Une position récente évite d'attendre le GPS : 2 décimales ne bougent pas en 10 min à pied. */
const LAST_POSITION_MAX_AGE_MS = 10 * 60_000;
// ponytail: marges système approchées. react-native-safe-area-context ne compile pas sous
// Windows (chemins trop longs pour ninja). À remettre avec HeroUI (B4), qui l'exige, et
// indispensable pour iOS : encoche et barre d'accueil ne sont pas gérées ici.
const INSET_TOP = Platform.OS === 'android' ? (NativeStatusBar.currentHeight ?? 0) : 48;
const INSET_BOTTOM = 48;

/** La dernière réponse, ou `null` dès qu'un appel échoue : jamais de réponse ancienne à l'écran. */
function useRain(place: Place | null) {
  const [rain, setRain] = useState<Rain | null>(null);
  const [failed, setFailed] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [now, setNow] = useState(() => new Date());
  const latest = useRef(0);

  const load = useCallback(async () => {
    if (place === null) return;
    const call = ++latest.current;
    const abort = new AbortController();
    const timer = setTimeout(() => abort.abort(), TIMEOUT_MS);
    try {
      const url = `${API_URL}/v1/rain?lat=${place.lat}&lon=${place.lon}`;
      const res = await fetch(url, { signal: abort.signal });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body: Rain = await res.json();
      // Une réponse pour l'ancien lieu ne doit pas s'afficher sous le nouveau.
      if (call !== latest.current) return;
      setRain(body);
      setFailed(false);
    } catch {
      if (call !== latest.current) return;
      setRain(null);
      setFailed(true);
    } finally {
      clearTimeout(timer);
      setNow(new Date());
    }
  }, [place]);

  const refresh = useCallback(async () => {
    setRefreshing(true);
    await load();
    setRefreshing(false);
  }, [load]);

  useEffect(() => {
    setRain(null);
    setFailed(false);
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

type Located = 'ok' | 'denied' | 'blocked' | 'failed';

/** Le lieu actif : la position au démarrage, ou la ville choisie. Premier plan seulement (D5). */
function usePlace() {
  // ponytail: la ville choisie n'est pas gardée au redémarrage. Stockage local avec les réglages (B4).
  const [place, setPlace] = useState<Place | null>(null);
  const [locating, setLocating] = useState(true);

  const locate = useCallback(async (): Promise<Located> => {
    setLocating(true);
    try {
      const permission = await Location.requestForegroundPermissionsAsync();
      if (!permission.granted) return permission.canAskAgain ? 'denied' : 'blocked';
      const recent = await Location.getLastKnownPositionAsync({
        maxAge: LAST_POSITION_MAX_AGE_MS,
      });
      const { coords } =
        recent ??
        (await Location.getCurrentPositionAsync({ accuracy: Location.Accuracy.Balanced }));
      const lat = round2(coords.latitude);
      const lon = round2(coords.longitude);
      const [address] = await Location.reverseGeocodeAsync({ latitude: lat, longitude: lon }).catch(
        () => [],
      );
      setPlace({ name: address?.city ?? address?.district ?? 'Ma position', lat, lon });
      return 'ok';
    } catch {
      return 'failed';
    } finally {
      setLocating(false);
    }
  }, []);

  return { place, setPlace, locating, locate };
}

const LOCATE_NOTICE: Record<Exclude<Located, 'ok'>, string> = {
  denied: 'Sans ta position, cherche une ville',
  blocked: 'Position refusée dans les réglages. Cherche une ville',
  failed: 'Position introuvable. Cherche une ville',
};

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
  const { place, setPlace, locating, locate } = usePlace();
  const { rain, failed, refreshing, refresh, now } = useRain(place);
  const [sheetOpen, setSheetOpen] = useState(false);
  const [notice, setNotice] = useState<string>();

  const findMe = useCallback(async () => {
    const located = await locate();
    if (located === 'ok') {
      setNotice(undefined);
      setSheetOpen(false);
      return;
    }
    setNotice(LOCATE_NOTICE[located]);
    setSheetOpen(true);
  }, [locate]);

  useEffect(() => {
    findMe();
  }, [findMe]);

  const onLocate = useCallback(() => {
    // Android et iOS ne redemandent plus : seuls les réglages peuvent rendre la permission.
    if (notice === LOCATE_NOTICE.blocked) Linking.openSettings();
    else findMe();
  }, [findMe, notice]);

  const reply =
    place === null
      ? { lead: locating ? 'Position en cours' : 'Choisis un lieu' }
      : answer(rain, now);
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
      <Pressable
        accessibilityHint="Changer de lieu"
        accessibilityRole="button"
        onPress={() => setSheetOpen(true)}
        style={styles.place}
      >
        <Text numberOfLines={1} style={[type.detail, { color: colors.text }]}>
          {place?.name ?? 'Choisir un lieu'}
        </Text>
      </Pressable>

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
      <PlaceSheet
        insetTop={INSET_TOP}
        locating={locating}
        notice={notice}
        onClose={() => setSheetOpen(false)}
        onLocate={onLocate}
        onPick={(picked) => {
          setPlace({ name: picked.name, lat: picked.lat, lon: picked.lon });
          setNotice(undefined);
          setSheetOpen(false);
        }}
        visible={sheetOpen}
      />
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
  place: { minHeight: size.row, justifyContent: 'center' },
  answer: { flex: 1, justifyContent: 'center', paddingVertical: space.block },
  bars: { flexDirection: 'row', alignItems: 'flex-end', height: band.height, gap: space.bar },
  barSlot: { flex: 1 },
  baseline: { height: StyleSheet.hairlineWidth },
  hours: { flexDirection: 'row', justifyContent: 'space-between', marginTop: space.s },
  source: { marginTop: space.m },
  tabular: { fontVariant: ['tabular-nums'] },
});
