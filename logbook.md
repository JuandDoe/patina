# Logbook — Patina (passerelle de paiement Monero, Rust)

> Journal de bord du projet. But : garder une trace honnête de ce que je construis,
> de ce qui coince, de ce que j'apprends, et des retours de revue. On avance
> **par briques** : chaque brique doit compiler, tourner, et être notée ici avant
> de passer à la suivante.

---

## Comment je tiens ce journal

Une **entrée par session** de travail. Je remplis le modèle ci-dessous à chaud.
La règle : être franc sur ce qui a foiré ou ce que j'ai fait "à l'arrache" — c'est
là qu'est l'apprentissage, et c'est ce que la revue exploite. Embellir ne sert à rien.

Trois sections font le cœur de chaque entrée :

- **Ce qui a coincé** → mes vrais points de friction.
- **Décisions techniques** → ce que j'ai choisi *et pourquoi* (mon raisonnement, même bancal).
- **Revue du senior** → le retour critique + les corrections de cap.

---

## Roadmap

Logique d'amélioration : on part d'un truc minuscule qui marche, et on durcit
brique par brique jusqu'à un vrai service. Chaque phase est livrable seule.

- [x] **Phase 0 — Infra stagenet** : nœud (distant) + `monero-wallet-rpc` qui répond, fonds de test reçus.
- [x] **Phase 1 — v0 CLI** : générer une sous-adresse, détecter le paiement, attendre les confirmations.
- [x] **Phase 2 — Persistance** : couche SQLite + câblage au cycle de vie (sauve la vraie facture, marque `paid` à la confirmation) + tests hors-ligne. *Reporté : reprendre les `pending` au redémarrage → naturel avec l'API/le scanner.*
- [ ] **Phase 3/4 — Async** (tokio + axum) : scanner de fond sur toutes les `pending` (+ reprise au démarrage), puis API HTTP pour créer une facture / consulter son statut. Machine à états, idempotence.
- [ ] **Phase 5 — Webhooks** : notifier le marchand, signés + retries.
- [ ] **Phase 6 — Packaging** : Docker, config (env/fichier — dont le chemin de la DB), logs/observabilité.
- [ ] **Phase 7 — Confidentialité** : nœud auto-hébergé, Tor, *threat model* documenté.
- [ ] **Phase 8 — Qualité** : tests d'intégration, CI, README sérieux.
- [ ] **Niveau 2 (bonus)** : scan direct via la lib `monero-wallet` → internes Monero, rampe vers Cuprate.

---

## Modèle d'entrée (à copier-coller pour chaque session)

```
## Phase X — <titre> — <date>

**Objectif de la session :**

**Ce que j'ai fait :**

**Ce qui a coincé (honnête) :**

**Ce que j'ai appris :**

**Décisions techniques (et pourquoi) :**

**Revue du senior :**

**Prochaine étape :**
```

---

## Phase 0 — Mise en place de l'infra stagenet — *11/06/2026*

**Objectif de la session :**
Avoir `monero-wallet-rpc` qui tourne sur stagenet, connecté à un nœud, avec des
fonds de test, et confirmer que le RPC répond.

**Ce que j'ai fait :**

- Récupéré les binaires officiels Monero v0.18.5 (`monerod`, `monero-wallet-cli`, `monero-wallet-rpc`).
- Ajouté le dossier au `PATH`.
- Créé un wallet stagenet et lancé `monero-wallet-rpc` (port `38083`, `--disable-rpc-login`, en local).
- Récupéré des fonds via le faucet stagenet.
- Vérifié le RPC à la main (`create_address`, `get_balance`).

**Ce qui a coincé (honnête) :**

1. **PATH via copié-collé Stack Overflow.** J'ai collé dans `~/.bashrc` :
   `export PATH="/home/ant/project-monero/monero-x86_64-linux-gnu-v0.18.5.0:$PATH`
   sans trop comprendre, juste pour que ça marche.
nb : 13/06/2026, j'ai relu le poste stackoverflow, la syntaxe est claire
2. **Nœud stagenet fourni HS.** Le nœud `stagenet.community.rino.io:38081` ne répondait
   pas. J'ai cherché une liste de nœuds (`xmr.ditatompel.com/remote-nodes`) et basculé
   sur `node2.monerodevs.org:38089`, qui marche.

**Nœud utilisé actuellement :** `node2.monerodevs.org:38089` *(stagenet, port 38089)*

**Ce que j'ai appris :**

- La différence entre le **nœud** (la blockchain, port 38089 ici) et le **wallet-rpc**
  (mon wallet, port 38083 en local). Mon code parlera au wallet-rpc, pas au nœud.
- Les nœuds publics tombent — on ne peut pas en dépendre sérieusement.
- Nb : 13/06/2026 Je pense que ça ne change rien puisque mon wallet rpc contacte le noeud, par contre j'imagine qu'on peut donner une liste de noeud public au wallet rpc pour être plus resilient

**Décisions techniques (et pourquoi) :**

- Nœud **distant** pour le v0, pour éviter de synchroniser plusieurs Go.
  Conscient que c'est temporaire (cf. Phase 7).

**Revue du senior :**

- ✅ **Le bon réflexe de la session : le swap de nœud.** Diagnostiquer une dépendance
  morte dès la première heure et trouver soi-même une alternative qui marche, c'est
  *exactement* le métier. Et tu viens de vivre, en vrai, pourquoi "héberger son propre
  nœud" est sur la roadmap (Phase 7) : fiabilité **et** confidentialité. Garde ça en tête.
- ⚠️ **Le PATH versionné va te claquer dans les doigts.** Ton chemin contient le numéro
  de version (`...-v0.18.5.0`). À la prochaine mise à jour de Monero, ce dossier
  n'existera plus → `monerod` disparaît du PATH sans message d'erreur clair, et tu
  perds 30 min à comprendre pourquoi. **Mieux :** un lien symbolique non versionné, p.ex.
  `ln -s /home/ant/project-monero/monero-x86_64-linux-gnu-v0.18.5.0 ~/monero`, puis tu
  mets `~/monero` dans le PATH. Mise à jour = tu repointes le lien, le PATH ne bouge jamais.
  - Done + remboursement dette path
- ⚠️ **Vérifie le guillemet fermant.** La ligne que tu as collée n'a pas son `"` de fin
  (`...:$PATH` au lieu de `...:$PATH"`). Une guillemet ouverte non fermée casse le parsing
  du `.bashrc`. Si ton terminal s'ouvre normalement, c'est probablement juste une coquille
  - c'étaiyt bien juste une coquille de copier/collé
  recopiée ; sinon, c'est ton bug. À checker.
- 🧠 **Habitude à prendre, pas une faute :** comprendre une ligne *avant* de la coller,
  surtout quand elle touche au shell, au PATH, ou qu'elle commence par `sudo`. Ici c'était
  bénin. Un jour ça ne le sera pas. Le copié-collé qui marche est une dette, pas une dette nulle.

**Prochaine étape :** Phase 1 — écrire le v0 qui crée une sous-adresse et boucle jusqu'au paiement.

---

## Phase 1 — v0 CLI : détecter un paiement — *12–13/06/2026*

**Objectif de la session :**
Un binaire qui crée une sous-adresse de facture, l'affiche, surveille le wallet-rpc,
et ne déclare "PAYÉ" qu'une fois le montant reçu **ET** suffisamment confirmé.

**Ce que j'ai fait :**

- **Exo 1 — arguments CLI :** `account_index` et `expected_xmr` viennent maintenant de
  la ligne de commande (`cargo run -- 0 0.1`), parsés à la main depuis `env::args()`
  (pas de crate, pour apprendre).
- **Exo 2 — typage :** remplacé l'indexation `Value` de `create_address` par une struct
  `CreatedAddress { address, address_index }` peuplée via `#[derive(Deserialize)]` + `serde_json::from_value`.
- **Exo 3 — confirmations :** extraction du **minimum** des `confirmations` sur le tableau
  `in`, seuil en const, et boucle à 4 états (rien / partiel / reçu mais pas assez confirmé /
  payé+confirmé → `break`).
- **Dette soldée :** transferts désérialisés proprement en `Vec<Transfer>` typé (struct
  `Transfer` + `GetTransfers` avec `#[serde(rename="in", default)]`), `received`/`min_conf`
  calculés en deux lignes (`iter().map().sum()` / `.min()`).

**Ce qui a coincé (honnête) :**

1. **const vs runtime (~2 h).** J'ai voulu mettre les arguments CLI dans des `const`.
   Ça ne compile pas : une `const` veut une valeur connue à la **compilation**, or les
   args n'existent qu'à l'**exécution**. → `let`, et parser (les args arrivent en `String`).
2. **serde, l'enveloppe.** Je ne savais pas si ma struct devait décrire toute la réponse
   JSON ou juste l'intérieur. En fait `rpc_call` a déjà retiré l'enveloppe (`result`),
   donc ma struct décrit l'objet intérieur. Confusion aussi sur "une struct ou deux"
   (→ une struct, deux champs) et sur le rôle de `derive` vs `from_value`.
3. **Option<u64> sur `min_conf`.** Voulu caster en `u8` — impossible : on ne caste pas
   un `Option`. Confusion `None` (aucun transfert) vs `Some(0)` (transfert à 0 conf).
   Et j'avais inversé la condition (déclarer payé quand les confirmations étaient basses).

**Ce que j'ai appris :**

- **`const` = compile-time, `let` = runtime.** Les args CLI sont du texte → `.parse()`,
  qui renvoie un `Result` (peut échouer) → `?`.
- **serde = pont JSON ↔ types.** `#[derive(Deserialize)]` *génère* (à la compilation)
  l'implémentation qui sait remplir MA struct ; `from_value` est le moteur générique qui
  s'en sert (à l'exécution). Les champs JSON en trop sont ignorés. Cascade : désérialiser
  `GetTransfers` remplit ses `Vec<Transfer>` tout seul (chaque type de la cascade a son `derive`).
- **On ne caste pas un `Option`, on le dénoue** (`unwrap_or`, `if let Some`). `None` ≠ `Some(0)`.
- **"parse, don't validate"** : on concentre le doute à la frontière (le parse), ensuite
  on manipule du typé sûr.
- **Politique de confirmation** : un paiement vaut sa part la moins confirmée → on prend
  le `min` ; on attend un seuil (10 en prod, 2 en test).
- Une boucle `loop` rejoue **tout** son corps à chaque tour jusqu'à `break` (ou erreur via `?`).
- Itérateurs : `iter()` *prête* les éléments (`&Transfer`, la liste survit) ; `into_iter()`
  les *donne* (consomme). Une valeur est détruite quand son propriétaire meurt sans l'avoir transmise.

**Décisions techniques (et pourquoi) :**

- Parsing CLI à la main plutôt qu'un crate : comprendre avant d'abstraire.
- Structs typées plutôt que `Value` : fautes de frappe attrapées à la compilation, code
  lisible, doute levé une seule fois.
- Seuil en const `ULTIMATE_PART_CONFIRMATION: u64` ; type `u64` pour coller à ce que renvoie
  serde et éviter les casts.

**Revue du senior :**

- ✅ Lecture de la doc API et raisonnement *avant* de coder, décision du seuil avec source
  officielle, bon instinct de right-sizing. La diagnose des erreurs de compilation était
  correcte à chaque fois : le blocage venait d'un outil pas encore vu, pas d'un défaut de raisonnement.
- ⚠️ Mettre la struct au niveau module plutôt que dans `main`. *(nb : fait le 13/06/2026)*
- ⚠️ `None` vs `Some(0)` : soigner le libellé des commentaires. *(nb : fait le 13/06/2026)*
- 🧠 Timeboxing : après ~20-30 min sans progrès, changer de tactique.
- 🎯 Cap : les états "partiel" et "en attente de confirmations" sont les germes de la
  machine à états de la Phase 4.

**Prochaine étape :** Phase 2 — persistance (SQLite).

---

## Phase 2 — Persistance (SQLite) — *jusqu'au 19/06/2026*

**Objectif de la session :**
Donner à Patina une mémoire qui survit : stocker les factures dans SQLite, et brancher ça
sur le cycle de vie réel. ✅ Fait (la reprise des `pending` au redémarrage est reportée — voir dette).

**Ce que j'ai fait :**

- `rusqlite` (feature `bundled`). Couche SQL isolée dans `src/db.rs` ; `main` appelle en `db::…`.
- Table `invoices` (`CREATE TABLE IF NOT EXISTS`, `created_at TEXT DEFAULT CURRENT_TIMESTAMP`).
- Fonctions DB : `create_invoices_table`, `save_invoice` (INSERT paramétré, cast `as i64`),
  `invoice_paid` (UPDATE `pending` → `paid`), `get_invoice` (`query_row` + closure qui mappe
  une ligne vers une `struct Invoice`, cast `i64 as u64` au retour).
- **Décision de paiement extraite** dans une fonction *pure* `evaluate_payment(...) -> PaymentStatus`
  (enum à 4 états) ; la boucle fait un `match` dessus → décider / agir séparés.
- **Tests hors-ligne** : `evaluate_payment` (5 cas, dont sur-paiement) ; couche DB en base
  `open_in_memory` (save → get → `assert_eq!` ; `pending` → `paid` → `assert_eq!`).
- **Câblage réel** : on sauvegarde la *vraie* facture issue de `create_address` en `pending`,
  et on appelle `invoice_paid` à la confirmation. **Vérifié de bout en bout sur un vrai paiement de test.**

**Ce qui a coincé (honnête) :**

- **`u64` refusé par rusqlite 0.40** (SQLite n'a que des entiers signés `i64`) → cast aux
  deux frontières. Le n° de version dans le chemin de l'erreur (`rusqlite-0.40.1`) a donné la clé.
- **`query_map` au lieu de `query_row`** : `query_map` rend un itérateur (plusieurs lignes) ;
  pour lire une facture par son adresse unique il faut `query_row` (une ligne). L'autocomplétion
  m'a embarqué dans un truc trop complexe (`.nth(0).unwrap_or_else(...)`).
- **Struct `Invoice` en `i64` vs cast `as u64`** → conflit de types. Leçon : "ça compile ≠
  c'est juste". Le type suit le domaine (`u64`), on convertit *à la frontière*.
- **`CannotOpen` sur `sql/patina.db`** alors que le fichier existait : un chemin **relatif**
  est résolu depuis le répertoire de *lancement*, et `open` ne crée pas le dossier parent.
  → simplifié en `patina.db`.
- Vieux `if/else` laissé *sous* le `match` (double décision) → supprimé. `Ok(())` vs `Ok(());`.
  `mod db;` sans le fichier (`E0583`). `transaction` = mot réservé SQL → table `invoices`.
  Placeholders `?n` ≠ nombre de colonnes.

**Ce que j'ai appris :**

- **`query_row` (exactement une ligne) vs `query_map` (plusieurs → itérateur → `Vec`).**
  Les `?n` / `row.get(i)` sont des positions de **colonnes**, pas de lignes.
- **Le type suit le domaine, le stockage s'y plie** : struct en `u64`, cast `as i64`/`as u64`
  confiné à la frontière DB (les deux sens).
- `enum` + `match` pour des états mutuellement exclusifs ; `#[cfg(test)]` / `#[test]` /
  `assert_eq!` / `use super::*`. Un test **sans `assert`** ne prouve que "ça ne panique pas".
- **Le test EST l'usage** : pas besoin d'un faux appel dans `main` pour exercer une fonction.
- Modules : un fichier = un module à déclarer (`mod`), `pub` pour exposer, `use` par fichier.
  Tests **unitaires** dans le même fichier (accès au privé) ; dossier `tests/` = tests
  **d'intégration** (API publique seulement).
- "expected X, found Y" se lit "la cible veut X, tu donnes Y" — l'erreur est là où la valeur
  *atterrit*. Chemin relatif = résolu depuis le cwd. Vérifier la **version de la crate** dans
  un message d'erreur quand un exemple "ne marche pas chez moi".
- L'autocomplétion est géniale pour la *syntaxe*, pas pour l'*intention* — décider la structure
  soi-même d'abord, la laisser remplir ensuite.

**Décisions techniques (et pourquoi) :**

- Décision de paiement = fonction *pure* + `enum` → testable sans I/O.
- Couche DB isolée dans `db.rs` ; connexion détenue par `main`, prêtée en `&Connection`.
- `Invoice` en `u64` (domaine), conversions à la seule frontière SQLite.
- Chemin DB relatif `patina.db` pour l'instant (config = Phase 6).

**Dette technique restante (à rembourser) :**

- **Reprise au redémarrage** : au lancement on crée toujours une *nouvelle* facture ; on ne
  recharge pas les `pending`. → naturel quand Patina sera un service (Phase 3/4 : `query_map`
  sur les `pending`).
- Chemin DB en dur/relatif (Phase 6) ; les vieilles `pending` s'accumulent (pas de
  nettoyage/expiration) ; une seule facture à la fois ; mempool/0-conf (`"pool": true`) ;
  erreurs en `Box<dyn Error>` ; vérifier que `struct Invoice` est `pub` (type public renvoyé
  par `get_invoice`).

**Revue du senior :**

- ✅ **Vrai jalon** : la persistance est branchée de bout en bout et *vérifiée sur un paiement
  réel*, avec une couche DB testée hors-ligne. C'est un livrable, pas une démo.
- ✅ Grosse progression d'autonomie : tests DB écrits seul, doublon `if/else` repéré,
  `CannotOpen` diagnostiqué par le bon raisonnement (cwd, pas permissions).
- ✅ Leçon clé intégrée : "ça compile ≠ c'est juste" — type choisi par le domaine, pas par
  ce qui fait taire le compilateur.
- ⚠️ Garder les commits séparés par nature (refactor `evaluate_payment` / tests DB / câblage).
- 🎯 **Cap à anticiper** : l'archi actuelle (créer 1 facture, la surveiller dans une boucle
  *bloquante*) va changer — la boucle deviendra un **scanner de fond asynchrone** sur *toutes*
  les `pending`, et la création passera par une **API**. C'est là qu'arrive l'**async (tokio)** :
  le morceau le plus raide de Rust, on ira lentement.

**Prochaine étape :** Phase 3/4 — introduire l'async (tokio) : transformer la boucle bloquante
en scanner de fond sur les `pending` (ce qui règle aussi la reprise au redémarrage), puis
exposer une API HTTP (axum) pour créer une facture et consulter son statut.
