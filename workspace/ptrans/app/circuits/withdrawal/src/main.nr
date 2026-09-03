mod merkle_tree;
use merkle_tree::compute_merkle_root;
use std::hash::poseidon2_permutation;

/*
 * Noir работает по принципу:
 * «если вычисление завершается успешно, то генерируем доказательство».
 *
 * Поэтому обычно в самом конце схемы добавляется проверка через утверждение:
 * assert(computed_root == root).
 */

// Глубина дерева Меркла - поддерживает 2^20 = 1024 депозитов
global TREE_DEPTH: u32 = 20;

// --- Вспомогательные функции для хэширования (ДОЛЖНЫ БЫТЬ ПЕРЕД main) ---

// Хэширует 1 поле
fn hash_1(input: [Field; 1]) -> Field {
    let state = [input[0], 0, 0, 0];
    poseidon2_permutation(state)[0]
}

// Хэширует 2 поля
fn hash_2(input: [Field; 2]) -> Field {
    let state = [input[0], input[1], 0, 0];
    poseidon2_permutation(state)[0]
}

// Хэширует 3 поля
fn hash_3(input: [Field; 3]) -> Field {
    let state = [input[0], input[1], input[2], 0];
    poseidon2_permutation(state)[0]
}

/*
 * Схема приватного перевода (Private Transfer Circuit)
 *
 * Доказывает знание (nullifier, secret, amount) таких, что:
 * 1. commitment = Poseidon(nullifier, secret, amount) существует в дереве Меркла
 * 2. nullifier_hash = Poseidon(nullifier) соответствует публичному вводу
 * 3. Доказательство привязано к конкретному получателю (публичный ввод)
 *
 * fn main - Точка входа в ZK-схему.
 * Переменные с пометкой pub видны всем (публичные инпуты),
 * остальные — скрыты (приватные инпуты доказывающего).
 */
fn main(
    root: pub Field,
    nullifier_hash: pub Field,
    recipient: pub Field,
    amount: pub Field,
    nullifier: Field,
    secret: Field,
    merkle_proof: [Field; TREE_DEPTH],
    is_even: [bool; TREE_DEPTH],
) {
    // Вычисляет commitment (обязательство) путем хэширования трех приватных полей:
    // нуллификатора, секрета и суммы перевода.
    let commitment = hash_3([nullifier, secret, amount]);

    // Генерирует хэш нуллификатора.
    // Нуллификатор нужен, чтобы предотвратить повторную трату (Double Spending) монеты без раскрытия.
    let computed_nullifier_hash = hash_1([nullifier]);

    // Проверяет, совпадает ли вычисленный хэш с публично предоставленным nullifier_hash.
    // Если нет, доказательство не сгенерируется.
    assert(computed_nullifier_hash == nullifier_hash, "Invalid nullifier hash");

    // На основе вычисленного commitment (который выступает в роли листа дерева)
    // и приватного пути восстанавливает корень дерева Меркла.
    let computed_root = compute_merkle_root(commitment, merkle_proof, is_even);

    // Проверяет, что восстановленный корень совпадает с легитимным публичным корнем дерева root.
    // Это доказывает, что обязательство реально существует в системе.
    assert(computed_root == root, "Invalid Merkle proof");

    /*
    * Фиктивное использование переменной recipient.
    * Так как она объявлена как pub, она жестко вшивается в криптографический транскрипт доказательства,
    * гарантируя, что злоумышленник не перехватит доказательство и не подменит адрес получателя на свой.
    * recipient является публичным вводом, привязывающим доказательство к этому конкретному получателю
    */
    let _ = recipient;
}

/*
 * Вспомогательные функции
 */
// Дублирует логику создания commitment для тестов.
fn compute_commitment(nullifier: Field, secret: Field, amount: Field) -> Field {
    hash_3([nullifier, secret, amount])
}

// Дублирует логику создания хэша нуллификатора для тестов.
fn compute_nullifier_hash(nullifier: Field) -> Field {
    hash_1([nullifier])
}

/*
 * вычисляет хэши «пустых» поддеревьев для каждого уровня дерева Меркла
 * результат - массив zeros будет содержать:
 * zeros[0] = 0 (пустой лист)
 * zeros[1] = Hash(0, 0) (пустая пара листьев)
 * zeros[2] = Hash(zeros[1], zeros[1]) (пустая четверка листьев)
 * ... и так далее до 20-го уровня.
 */
fn compute_empty_tree_zeros() -> [Field; TREE_DEPTH] {
    let mut zeros: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current: Field = 0;

    for i in 0..TREE_DEPTH {
        zeros[i] = current;
        current = hash_2([current, current]);
    }

    zeros
}

#[test]
/*
 * Проверяет детерминированность хэша (одинаковые инпуты дают одинаковый результат)
 * и его чувствительность к изменению суммы (amount + 1).
 */
fn test_commitment_computation() {
    let nullifier = 12345;
    let secret = 67890;
    let amount = 1000000000; // 1 SOL in lamports
    let commitment = compute_commitment(nullifier, secret, amount);
    // Обязательство должно быть детерминированным
    let commitment2 = compute_commitment(nullifier, secret, amount);
    assert(commitment == commitment2);

    // Другая сумма должна приводить к другому обязательству
    let commitment3 = compute_commitment(nullifier, secret, amount + 1);
    assert(commitment != commitment3);
}

#[test]
/*
 * Гарантирует, что хэш нуллификатора уникален
 * и корректно меняется при изменении входного значения.
 */
fn test_nullifier_hash() {
    let nullifier = 12345;
    let hash1 = compute_nullifier_hash(nullifier);
    let hash2 = compute_nullifier_hash(nullifier);
    assert(hash1 == hash2);

    // Другой нуллификатор должен приводить к другому хэшу
    let hash3 = compute_nullifier_hash(nullifier + 1);
    assert(hash1 != hash3);
}

#[test]
/*
 * Тестирует коллизии.
 * Проверяет, что при одинаковых секретах, но разных суммах, получаются разные обязательства.
 */
fn test_commitment_uniqueness() {
    // Одинаковые нуллификатор/секрет, но разные суммы должны приводить к разным обязательствам
    let nullifier = 99999;
    let secret = 88888;
    let amount1 = 100000000; // 0.1 SOL
    let amount2 = 200000000; // 0.2 SOL

    let commitment1 = compute_commitment(nullifier, secret, amount1);
    let commitment2 = compute_commitment(nullifier, secret, amount2);

    assert(commitment1 != commitment2);
}

#[test]
/*
 * Эмулирует полный цикл работы.
 * Генерирует корректные данные для нулевого индекса дерева
 * (где все соседи — это пустые дефолтные поддеревья zeros),
 * считает корень дерева и проверяет, что повторное вычисление сходится.
 */
fn test_generate_valid_inputs() {
    let nullifier: Field = 12345;
    let secret: Field = 67890;
    let amount: Field = 1000000000;
    let recipient: Field = 123456789;

    let commitment = compute_commitment(nullifier, secret, amount);
    let nullifier_hash = compute_nullifier_hash(nullifier);
    let zeros = compute_empty_tree_zeros();

    // Для листа с индексом 0 соседними элементами являются пустые поддеревья
    let merkle_proof = zeros;
    let is_even = [true; TREE_DEPTH];
    let root = merkle_tree::compute_merkle_root(commitment, merkle_proof, is_even);

    println("=== Данные для Prover.toml ===");
    println(f"root = \"{root}\"");
    println(f"secret = \"{secret}\"");
    println(f"amount = \"{amount}\"");

    println("is_even = [");
    for i in 0..TREE_DEPTH {
        print("1");
        if i < TREE_DEPTH - 1 {
            println(",");
        } else {
            println("");
        }
    }
    println("]");

    println("merkle_proof = [");
    for i in 0..TREE_DEPTH {
        let prove = zeros[i];
        print(f"\"{prove}\"");
        if i < TREE_DEPTH - 1 {
            println(",");
        } else {
            println("");
        }
    }
    println("]");

    println(f"nullifier = \"{nullifier}\"");
    println(f"nullifier_hash = \"{nullifier_hash}\"");
    println(f"recipient = \"{recipient}\"");
    println("=== Конец данных ===");

    let computed = merkle_tree::compute_merkle_root(commitment, merkle_proof, is_even);

    assert(computed == root);
}
