use ::std::hash::poseidon2_permutation;

/*
 * Вычисляет корень Меркла на основе листа и его доказательства (proof)
 * # Аргументы
 * leaf - Значение листа для проверки
 * path - Массив хэшей соседних элементов (сиблингов) вдоль пути к корню
 * is_even - Булев массив, указывающий, находится ли лист слева (четный)
 * или справа (нечетный) на каждом уровне
 *
 * # Возвращает
 * Вычисленный корень Меркла
 */
pub fn compute_merkle_root<let DEPTH: u32>(
    leaf: Field,
    path: [Field; DEPTH],
    is_even: [bool; DEPTH],
) -> Field {
    // значение начального листа, переменная будет обновляться на каждом уровне дерева
    let mut current = leaf;

    // Цикл, который проходит снизу вверх по всем уровням дерева от 0 до DEPTH - 1
    for i in 0..DEPTH {
        // Достает хэш соседнего элемента (сиблинга) для текущего уровня.
        let sibling = path[i];

        /*
         * Проверяет положение текущего узла.
         * Если is_even[i] истинно, текущий узел левый, а сосед — правый.
         * Иначе — наоборот. Это важно, так как hash(A, B) != hash(B, A).
         */
        let (left, right) = if is_even[i] {
            (current, sibling)
        } else {
            (sibling, current)
        };

        // Хэширует пару [left, right]
        // с помощью криптографической хэш-функции Poseidon2
        // для получения родительского узла.
        // Для хэширования пары используем Poseidon2 permutation
        // Передаем массив из 2 элементов, остальные дополняются нулями
        let state = [left, right, 0, 0];
        current = poseidon2_permutation(state)[0];
    }

    current
}
/*
 * Проверяет доказательство Меркла (Merkle proof)
 *
 * # Аргументы
 * leaf - Значение листа для проверки
 * root - Ожидаемый корень Меркла
 * path - Массив хэшей соседних элементов
 * is_even - Булев массив для направления пути
 *
 * # Возвращает
 * True, если доказательство валидно
 */
pub fn verify_merkle_proof<let DEPTH: u32>(
    leaf: Field,
    root: Field,
    path: [Field; DEPTH],
    is_even: [bool; DEPTH],
) -> bool {
    // Вычисление корня на основе предоставленного пути.
    let computed_root = compute_merkle_root(leaf, path, is_even);

    // Сравнивает полученный корень с эталонным и возвращает результат (true или false).
    computed_root == root
}
