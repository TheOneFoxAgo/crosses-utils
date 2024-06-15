# crosses-core

Полное описание игры ищите в книге **"Логика или фортуна"**. Ядро игры существует, чтобы позволить интересующимся заменять главные алгоритмы и структуры данных, не трогая игровую логику.

# Использование:

1. Пользователь реализует структуру игрового поля, а ядро предоставляет ему 2 игровые функции: `make_move()` и `cancel_move()`, а также 1 служебную: `init()`.
	- `make_move()` - сделать ход
	- `cancel_move()` - отменить ход
	- `init()` - проинициализировать поле: расставить все нужные флаги и активность. **ВНИМАНИЕ!** Функция `init()` ожидает, что все клетки изначально имеют "нулевые" активность флаги и состояние. (Закрашенные клетки мертвы, крестики не якори)
2. Пользователь инициализирует поле (С помощью `init()`).
3. Пользователь инициализирует `PlayerManager` (Специальная структура для организации порядка ходов)
4. Пользователь использует эти структуры в своей реализации игры (Методы `make_move()` и `cancel_move()`)
5. ???
6. Profit!!!

# Ключевые контракты

Чтобы можно было предоставить функции `make_move()` и `cancel_move()` надо удовлетворить следующие контракты для поля и его клетки.

## Клетка

Клетка - единица поля. Имеет 3+1 ключевых состояния: пуста, содержит крестик, закрашена, является границей. Трейт клетки называется CellHandle неспроста. Это намёк на то, что вам не обязательно отдавать ссылку на то, что действительно храниться в поле. Это может быть более умный объект, способный, например, обновлять счётчик доступных ходов игрока при действиях с клеткой.

### Пустая клетка

Должна иметь следующие свойства:
1. Тип
2. Активность (Может ли игрок взаимодействовать с клеткой. Для каждого игрока, должна храниться своя, индивидуальная активность. Рекомендую использовать `bitfield` или что-то подобное)

### Крестик

1. Тип
2. Игрок (У каждого игрока свой цвет, а у цвета - игрок, поэтому слова игрок и цвет в контексте проекта - синонимы)
3. Активность (Естественно, для каждого игрока своя. Поскольку игрок не может взаимодействовать с крестиком своего цвета, ядро не читает активность крестика для своего же цвета. Можете её использовать для своих нужд, например для следующего пункта. Хочу обратить внимание, что несмотря на то, что поле активности для игрока своего же цвета не используется, по смыслу крестик не является активным для своего игрока.)
4. Флаг "Якоря" (`bool`. Используется для для уменьшения перепроверок.)

### Закрашенная клетка

1. Тип
2. Игрок
3. Предыдущий игрок (Это нужно в для отмены ходов)
4. Состояние: мёртвая, живая, у якоря (А значит живая), отмечена (Нужно для проверки).

### Граница

1. Тип. Ну и всё на этом.

## Правила преобразования типов
Клетки должны преобразовываться из одного типа в другой по этим правилам:

1. Пустая клетка в крестик:
	- Тип меняется на крестик
	- Активность не изменяется за исключением активности игрока, поставившего крестик. Её можно изменить
	- Флаг якоря --- `false`
2. Крестик в пустую клетку:
	- Тип меняется на пустую клетку
	- Активность не изменяется за исключением активности игрока, поставившего крестик. Она должна быть "активной"
3. Крестик в закрашенную клетку:
	- Тип меняется на закрашенную клетку
	- Игрок становится предыдущим игроком
	- Игрок меняется на нового
	- Состояние --- "Живая"
4. Закрашенная клетка в крестик:
	- Тип меняется на закрашенную клетку
	- Предыдущий игрок становится игроком
	- Активность --- нулевая
	- Якорь --- `false`
	
## Поле

Само игровое поле должно реализовывать следующие методы:
1. Получение клетки по индексу.
2. Получение всех индексов клеток вокруг.
3. Обход цепочки закрашенных клеток (И смежных с ними). На вход этому методу подаются: начальная клетка обхода (Гарантированно закрашенная) и стратегия. Стратегией является структура с двумя методами:
	- `is_traversed(cell)` - Возвращает прошли ли эту клетку или ещё нет.
	- `process(board, index)` - Обрабатывает клетку. Если был возвращён `core::ops::ControlFlow::Break`, то необходимо вернуть индекс клетки, для которой это произошло. Иначе None.

 	На возвращаемое значение нужно соответствующим образом реагировать. Все пройденные клетки должны быть обработаны (Для каждой должен быть вызван метод `process`).


