# Logbook — Passerelle de paiement Monero (Rust)

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
- [ ] **Phase 1 — v0 CLI** : générer une sous-adresse, détecter le paiement dessus. *(en cours)*
- [ ] **Phase 2 — Persistance** : stocker les factures (SQLite), survivre à un redémarrage.
- [ ] **Phase 3 — API HTTP** (axum) : créer une facture, consulter son statut.
- [ ] **Phase 4 — Scanner de fond** (tokio) : machine à états, confirmations, idempotence.
- [ ] **Phase 5 — Webhooks** : notifier le marchand, signés + retries.
- [ ] **Phase 6 — Packaging** : Docker, config (env/fichier), logs/observabilité.
- [ ] **Phase 7 — Confidentialité** : nœud auto-hébergé, Tor, *threat model* documenté.
- [ ] **Phase 8 — Qualité** : tests, CI, README sérieux.
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
2. **Nœud stagenet fourni HS.** Le nœud `stagenet.community.rino.io:38081` ne répondait
   pas. J'ai cherché une liste de nœuds (`xmr.ditatompel.com/remote-nodes`) et basculé
   sur `node2.monerodevs.org:38089`, qui marche.

**Nœud utilisé actuellement :** `node2.monerodevs.org:38089` *(stagenet, port 38089)*

**Ce que j'ai appris :**
- La différence entre le **nœud** (la blockchain, port 38089 ici) et le **wallet-rpc**
  (mon wallet, port 38083 en local). Mon code parlera au wallet-rpc, pas au nœud.
- Les nœuds publics tombent — on ne peut pas en dépendre sérieusement.

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
- ⚠️ **Vérifie le guillemet fermant.** La ligne que tu as collée n'a pas son `"` de fin
  (`...:$PATH` au lieu de `...:$PATH"`). Une guillemet ouverte non fermée casse le parsing
  du `.bashrc`. Si ton terminal s'ouvre normalement, c'est probablement juste une coquille
  recopiée ; sinon, c'est ton bug. À checker.
- 🧠 **Habitude à prendre, pas une faute :** comprendre une ligne *avant* de la coller,
  surtout quand elle touche au shell, au PATH, ou qu'elle commence par `sudo`. Ici c'était
  bénin. Un jour ça ne le sera pas. Le copié-collé qui marche est une dette, pas une dette nulle.

**Prochaine étape :** Phase 1 — écrire le v0 qui crée une sous-adresse et boucle jusqu'au paiement.

---

## Phase 1 — v0 CLI : détecter un paiement — *(en cours)*

**Objectif de la session :**
Un binaire qui : (1) crée une sous-adresse de facture, (2) l'affiche, (3) boucle en
interrogeant le wallet-rpc, (4) affiche "PAYÉ" quand les fonds arrivent dessus.

**Code de départ :** fourni par le senior (`monero-gateway-v0/`), compile et tourne.

**Simplifications ASSUMÉES (= dette technique à rembourser plus tard) :**
- On manipule du `serde_json::Value` non typé (indexation `["..."]`). → *Refactor 1 :*
  remplacer par des structs typées avec `#[derive(Deserialize)]`. C'est le premier
  exercice de "vrai Rust".
- Montant et `account_index` codés en dur, une seule facture à la fois.
- Pas de persistance : si le programme redémarre, il oublie la facture. → Phase 2.
- On ne regarde que les transferts **en chaîne** (`"in"`). Détecter le 0-conf
  (mempool, via `"pool": true`) viendra avec la notion de **confirmations**.
- Erreurs gérées avec `Box<dyn Error>` + `?` (simple). → un vrai type d'erreur plus tard.

**Exercices pour m'approprier le code (avant de me laisser ajouter des features) :**
1. Sortir le montant attendu et l'`account_index` dans des constantes claires (déjà en haut) — puis les rendre paramétrables (argument CLI).
2. Remplacer l'indexation `Value` de `create_address` par une struct `CreatedAddress { address: String, address_index: u64 }`.
3. Afficher le **nombre de confirmations** du paiement (le champ existe dans la réponse `get_transfers`).

1. Je ne touche pas a account_index qui est déjà une constante claire , je modifie expected_xmr en const EXPECTED_XMR dans main()

J""ai trouvé un tutoriel officiel pour que mon programme accepte des arguments CLI, j'opte pour cette option pour apprendre, au lieux de me reposer sur un crate
](https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html)"

Mon aproche de const était confuse, j'ai retiré les const sur les arguments CLI. Vu avec Claude. Comme les variables account_index et et xmr_expected sont définies désormais via des arguments CLI elles ne sont connu qu'a l'execution du programme et non a la compilation. Il convient également de parser les arguments puisqu'ils sont récupérés initialement sous forme de String

2.

3
Je vois effectivement que confirmaation n'est pas a la racine mais dans un array de transfert appellé "in"
Il faut que la partie du paiement la plus tardive est un certain nombre de confirmation. J'appellerai la variable ultimate_part_confirmation
[text](<https://www.getmonero.org/get-started/accepting/#:~:text=Wait%20until%20the%20payment%20has,can%20spend%20the%20funds.).>)
getmonero.org parle de 10 confirmation minimum, je partirais sur ça. D'ailleurs je pense qu'on pourrait en faire une const. Pour des raisons de test je mettrais la valeur a 2

- } else {
            let got = received as f64 / ATOMIC_UNITS_PER_XMR as f64;
            println!("PAYÉ : {got} XMR reçus. Facture réglée.");

            il faudrait checkeer ici mais je suppose qu'on va se retrouver avec un json contenant plusieurs tableau in si on regle  en pluisuers fois, je vois ce que tu essayais de me dire. Mais la tu vois je manque encore de l'intuition sur ou chercher /me debrouiller comme un grand avec la syntax.
            Je vais probablement devoir attendre  le log             println!("PAYÉ : {got} XMR reçus. Facture réglée."); puis commencver a ce moment la a trouver le le dernier tableau in
            Je vois bien un truc comme faire une boucle sur les tableau in jusqu'au dernier puis rentrer dnas le dernier et boucler jusqua ce que confirmation = 10. mais la c'est tout flou. je suis incapable d'écrire ça seul


**Ce qui a coincé :** *(à remplir quand je code)*

**Ce que j'ai appris :** *(à remplir)*

**Revue du senior :** *(à remplir après que je lui montre mon code modifié)*

**Prochaine étape :** *(à définir)*
