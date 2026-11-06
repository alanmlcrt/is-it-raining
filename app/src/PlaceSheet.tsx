import { useEffect, useState } from 'react';
import { FlatList, Modal, Pressable, StyleSheet, Text, TextInput, View } from 'react-native';

import { type City, MIN_QUERY, parseCities, type Place, searchUrl } from './place';
import { radius, size, space, type, useColors } from './theme';

const DEBOUNCE_MS = 300;

type Search = { cities: City[]; message?: string };

/** Open-Meteo Geocoding, appelé directement depuis l'app (D7). */
function useCitySearch(query: string): Search {
  const [search, setSearch] = useState<Search>({ cities: [] });

  const tooShort = query.trim().length < MIN_QUERY;

  useEffect(() => {
    if (tooShort) return;
    const abort = new AbortController();
    const timer = setTimeout(async () => {
      try {
        const res = await fetch(searchUrl(query), { signal: abort.signal });
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const cities = parseCities(await res.json());
        setSearch({ cities, message: cities.length === 0 ? 'Aucune ville trouvée' : undefined });
      } catch {
        if (!abort.signal.aborted) setSearch({ cities: [], message: 'Recherche indisponible' });
      }
    }, DEBOUNCE_MS);
    return () => {
      clearTimeout(timer);
      abort.abort();
    };
  }, [query, tooShort]);

  return tooShort ? { cities: [] } : search;
}

type Props = {
  visible: boolean;
  notice?: string;
  insetTop: number;
  locating: boolean;
  onClose: () => void;
  onLocate: () => void;
  onPick: (place: Place) => void;
};

export function PlaceSheet({
  visible,
  notice,
  insetTop,
  locating,
  onClose,
  onLocate,
  onPick,
}: Props) {
  const colors = useColors();
  const [query, setQuery] = useState('');
  const { cities, message } = useCitySearch(query);

  return (
    <Modal visible={visible} animationType="slide" onRequestClose={onClose}>
      <View style={[styles.sheet, { backgroundColor: colors.bg, paddingTop: insetTop + space.m }]}>
        <View style={styles.top}>
          <Text accessibilityRole="header" style={[type.lead, { color: colors.text }]}>
            Lieu
          </Text>
          <Pressable accessibilityRole="button" onPress={onClose} style={styles.close}>
            <Text style={[type.detail, { color: colors.text2 }]}>Fermer</Text>
          </Pressable>
        </View>

        <TextInput
          accessibilityLabel="Chercher une ville"
          autoCorrect={false}
          onChangeText={setQuery}
          placeholder="Chercher une ville"
          placeholderTextColor={colors.text3}
          returnKeyType="search"
          style={[
            type.detail,
            styles.input,
            { color: colors.text, borderColor: colors.hairline },
          ]}
          value={query}
        />

        {notice && <Text style={[type.detail, styles.note, { color: colors.text2 }]}>{notice}</Text>}

        <Pressable
          accessibilityRole="button"
          accessibilityState={{ busy: locating, disabled: locating }}
          disabled={locating}
          onPress={onLocate}
          style={styles.row}
        >
          <Text style={[type.detail, { color: locating ? colors.text2 : colors.text }]}>
            {locating ? 'Position en cours' : 'Ma position'}
          </Text>
        </Pressable>

        <FlatList
          data={cities}
          keyboardShouldPersistTaps="handled"
          keyExtractor={(city) => String(city.id)}
          ListEmptyComponent={
            message ? (
              <Text style={[type.detail, styles.note, { color: colors.text2 }]}>{message}</Text>
            ) : undefined
          }
          renderItem={({ item }) => (
            <Pressable
              accessibilityLabel={`${item.name}, ${item.region}`}
              accessibilityRole="button"
              onPress={() => onPick(item)}
              style={styles.row}
            >
              <Text style={[type.detail, { color: colors.text }]}>{item.name}</Text>
              <Text style={[type.caption, { color: colors.text2 }]}>{item.region}</Text>
            </Pressable>
          )}
        />
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  sheet: { flex: 1, paddingHorizontal: space.edge },
  top: { flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between' },
  close: { minHeight: size.row, justifyContent: 'center' },
  input: {
    minHeight: size.row,
    marginTop: space.m,
    paddingHorizontal: space.m,
    borderWidth: StyleSheet.hairlineWidth,
    borderRadius: radius.row,
  },
  note: { marginTop: space.m },
  row: { minHeight: size.row, justifyContent: 'center', paddingVertical: space.s },
});
