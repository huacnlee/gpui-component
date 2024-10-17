use std::time::{self, Duration};

use fake::{Fake, Faker};
use gpui::{
    div, impl_actions, px, AnyElement, AppContext, Edges, InteractiveElement, IntoElement,
    ParentElement, Pixels, Render, SharedString, Styled, Timer, View, ViewContext,
    VisualContext as _, WindowContext,
};
use serde::Deserialize;
use ui::{
    button::{Button, ButtonStyled},
    checkbox::Checkbox,
    context_menu::ContextMenuExt,
    h_flex,
    indicator::Indicator,
    input::{InputEvent, TextInput},
    label::Label,
    popup_menu::PopupMenuExt,
    prelude::FluentBuilder as _,
    table::{ColFixed, ColSort, Table, TableDelegate, TableEvent},
    v_flex, Selectable, Size, StyleSized as _,
};

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct ChangeSize(Size);

impl_actions!(table_story, [ChangeSize]);

#[derive(Clone, Debug, Default)]
struct Stock {
    id: usize,
    symbol: String,
    name: String,
    price: f64,
    change: f64,
    change_percent: f64,
    volume: f64,
    turnover: f64,
    market_cap: f64,
    ttm: f64,
    five_mins_ranking: f64,
    th60_days_ranking: f64,
    year_change_percent: f64,
    bid: f64,
    bid_volume: f64,
    ask: f64,
    ask_volume: f64,
    open: f64,
    prev_close: f64,
    high: f64,
    low: f64,
    turnover_rate: f64,
    rise_rate: f64,
    amplitude: f64,
    pe_status: f64,
    pb_status: f64,
    volume_ratio: f64,
    bid_ask_ratio: f64,
    latest_pre_close: f64,
    latest_post_close: f64,
    pre_market_cap: f64,
    pre_market_percent: f64,
    pre_market_change: f64,
    post_market_cap: f64,
    post_market_percent: f64,
    post_market_change: f64,
    float_cap: f64,
    shares: i64,
    shares_float: i64,
    day_5_ranking: f64,
    day_10_ranking: f64,
    day_30_ranking: f64,
    day_120_ranking: f64,
    day_250_ranking: f64,
}

impl Stock {
    fn random_update(&mut self) {
        self.price = (-300.0..999.999).fake::<f64>();
        self.change = (-0.1..5.0).fake::<f64>();
        self.change_percent = (-0.1..0.1).fake::<f64>();
        self.volume = (-300.0..999.999).fake::<f64>();
        self.turnover = (-300.0..999.999).fake::<f64>();
        self.market_cap = (-1000.0..9999.999).fake::<f64>();
        self.ttm = (-1000.0..9999.999).fake::<f64>();
        self.five_mins_ranking = self.five_mins_ranking * (1.0 + (-0.2..0.2).fake::<f64>());
        self.bid = self.price * (1.0 + (-0.2..0.2).fake::<f64>());
        self.bid_volume = (100.0..1000.0).fake::<f64>();
        self.ask = self.price * (1.0 + (-0.2..0.2).fake::<f64>());
        self.ask_volume = (100.0..1000.0).fake::<f64>();
        self.bid_ask_ratio = self.bid / self.ask;
        self.volume_ratio = self.volume / self.turnover;
        self.high = self.price * (1.0 + (0.0..1.5).fake::<f64>());
        self.low = self.price * (1.0 + (-1.5..0.0).fake::<f64>());
    }
}

fn random_stocks(size: usize) -> Vec<Stock> {
    (0..size)
        .map(|id| Stock {
            id,
            symbol: Faker.fake::<String>(),
            name: Faker.fake::<String>(),
            change: (-100.0..100.0).fake(),
            change_percent: (-1.0..1.0).fake(),
            volume: (0.0..1000.0).fake(),
            turnover: (0.0..1000.0).fake(),
            market_cap: (0.0..1000.0).fake(),
            ttm: (0.0..1000.0).fake(),
            five_mins_ranking: (0.0..1000.0).fake(),
            th60_days_ranking: (0.0..1000.0).fake(),
            year_change_percent: (-1.0..1.0).fake(),
            bid: (0.0..1000.0).fake(),
            bid_volume: (0.0..1000.0).fake(),
            ask: (0.0..1000.0).fake(),
            ask_volume: (0.0..1000.0).fake(),
            open: (0.0..1000.0).fake(),
            prev_close: (0.0..1000.0).fake(),
            high: (0.0..1000.0).fake(),
            low: (0.0..1000.0).fake(),
            turnover_rate: (0.0..1.0).fake(),
            rise_rate: (0.0..1.0).fake(),
            amplitude: (0.0..1000.0).fake(),
            pe_status: (0.0..1000.0).fake(),
            pb_status: (0.0..1000.0).fake(),
            volume_ratio: (0.0..1.0).fake(),
            bid_ask_ratio: (0.0..1.0).fake(),
            latest_pre_close: (0.0..1000.0).fake(),
            latest_post_close: (0.0..1000.0).fake(),
            pre_market_cap: (0.0..1000.0).fake(),
            pre_market_percent: (-1.0..1.0).fake(),
            pre_market_change: (-100.0..100.0).fake(),
            post_market_cap: (0.0..1000.0).fake(),
            post_market_percent: (-1.0..1.0).fake(),
            post_market_change: (-100.0..100.0).fake(),
            float_cap: (0.0..1000.0).fake(),
            shares: (100000..9999999).fake(),
            shares_float: (100000..9999999).fake(),
            day_5_ranking: (0.0..1000.0).fake(),
            day_10_ranking: (0.0..1000.0).fake(),
            day_30_ranking: (0.0..1000.0).fake(),
            day_120_ranking: (0.0..1000.0).fake(),
            day_250_ranking: (0.0..1000.0).fake(),
            ..Default::default()
        })
        .collect()
}

struct Column {
    id: SharedString,
    name: SharedString,
    sort: Option<ColSort>,
}

impl Column {
    fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        sort: Option<ColSort>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            sort,
        }
    }
}

struct StockTableDelegate {
    stocks: Vec<Stock>,
    columns: Vec<Column>,
    size: Size,
    loop_selection: bool,
    col_resize: bool,
    col_order: bool,
    col_sort: bool,
    col_selection: bool,
    loading: bool,
    fixed_cols: bool,
    is_eof: bool,
}

impl StockTableDelegate {
    fn new(size: usize) -> Self {
        Self {
            size: Size::default(),
            stocks: random_stocks(size),
            columns: vec![
                Column::new("id", "ID", None),
                Column::new("symbol", "Symbol", Some(ColSort::Default)),
                Column::new("name", "Name", None),
                Column::new("price", "Price", Some(ColSort::Default)),
                Column::new("change", "Chg", Some(ColSort::Default)),
                Column::new("change_percent", "Chg%", Some(ColSort::Default)),
                Column::new("volume", "Volume", Some(ColSort::Default)),
                Column::new("turnover", "Turnover", Some(ColSort::Default)),
                Column::new("market_cap", "Market Cap", Some(ColSort::Default)),
                Column::new("ttm", "TTM", Some(ColSort::Default)),
                Column::new("five_mins_ranking", "5m Ranking", Some(ColSort::Default)),
                Column::new("th60_days_ranking", "60d Ranking", Some(ColSort::Default)),
                Column::new("year_change_percent", "Year Chg%", Some(ColSort::Default)),
                Column::new("bid", "Bid", Some(ColSort::Default)),
                Column::new("bid_volume", "Bid Vol", Some(ColSort::Default)),
                Column::new("ask", "Ask", Some(ColSort::Default)),
                Column::new("ask_volume", "Ask Vol", Some(ColSort::Default)),
                Column::new("open", "Open", Some(ColSort::Default)),
                Column::new("prev_close", "Prev Close", Some(ColSort::Default)),
                Column::new("high", "High", Some(ColSort::Default)),
                Column::new("low", "Low", Some(ColSort::Default)),
                Column::new("turnover_rate", "Turnover Rate", Some(ColSort::Default)),
                Column::new("rise_rate", "Rise Rate", Some(ColSort::Default)),
                Column::new("amplitude", "Amplitude", Some(ColSort::Default)),
                Column::new("pe_status", "P/E", Some(ColSort::Default)),
                Column::new("pb_status", "P/B", Some(ColSort::Default)),
                Column::new("volume_ratio", "Volume Ratio", Some(ColSort::Default)),
                Column::new("bid_ask_ratio", "Bid Ask Ratio", Some(ColSort::Default)),
                Column::new(
                    "latest_pre_close",
                    "Latest Pre Close",
                    Some(ColSort::Default),
                ),
                Column::new(
                    "latest_post_close",
                    "Latest Post Close",
                    Some(ColSort::Default),
                ),
                Column::new("pre_market_cap", "Pre Mkt Cap", Some(ColSort::Default)),
                Column::new("pre_market_percent", "Pre Mkt%", Some(ColSort::Default)),
                Column::new("pre_market_change", "Pre Mkt Chg", Some(ColSort::Default)),
                Column::new("post_market_cap", "Post Mkt Cap", Some(ColSort::Default)),
                Column::new("post_market_percent", "Post Mkt%", Some(ColSort::Default)),
                Column::new("post_market_change", "Post Mkt Chg", Some(ColSort::Default)),
                Column::new("float_cap", "Float Cap", Some(ColSort::Default)),
                Column::new("shares", "Shares", Some(ColSort::Default)),
                Column::new("shares_float", "Float Shares", Some(ColSort::Default)),
                Column::new("day_5_ranking", "5d Ranking", Some(ColSort::Default)),
                Column::new("day_10_ranking", "10d Ranking", Some(ColSort::Default)),
                Column::new("day_30_ranking", "30d Ranking", Some(ColSort::Default)),
                Column::new("day_120_ranking", "120d Ranking", Some(ColSort::Default)),
                Column::new("day_250_ranking", "250d Ranking", Some(ColSort::Default)),
            ],
            loop_selection: true,
            col_resize: true,
            col_order: true,
            col_sort: true,
            col_selection: true,
            fixed_cols: false,
            loading: false,
            is_eof: false,
        }
    }

    fn update_stocks(&mut self, size: usize) {
        self.stocks = random_stocks(size);
        self.is_eof = false;
        self.loading = false;
    }

    fn render_value_cell(&self, val: f64) -> AnyElement {
        let this = div()
            .h_full()
            .table_cell_size(self.size)
            .child(format!("{:.3}", val));
        // Val is a 0.0 .. n.0
        // 30% to red, 30% to green, others to default
        let right_num = ((val - val.floor()) * 1000.).floor() as i32;

        let this = if right_num % 3 == 0 {
            this.text_color(ui::red_600()).bg(ui::red_50().opacity(0.6))
        } else if right_num % 3 == 1 {
            this.text_color(ui::green_600())
                .bg(ui::green_50().opacity(0.6))
        } else {
            this
        };

        this.into_any_element()
    }
}

impl TableDelegate for StockTableDelegate {
    fn cols_count(&self) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &'_ AppContext) -> usize {
        self.stocks.len()
    }

    fn col_name(&self, col_ix: usize) -> SharedString {
        if let Some(col) = self.columns.get(col_ix) {
            col.name.clone()
        } else {
            "--".into()
        }
    }

    fn col_width(&self, col_ix: usize) -> Option<Pixels> {
        if let Some(_) = self.columns.get(col_ix) {
            Some(120.0.into())
        } else {
            None
        }
    }

    fn col_padding(&self, col_ix: usize) -> Option<Edges<Pixels>> {
        if col_ix >= 3 && col_ix <= 10 {
            Some(Edges::all(px(0.)))
        } else {
            None
        }
    }

    fn col_fixed(&self, col_ix: usize) -> Option<ui::table::ColFixed> {
        if !self.fixed_cols {
            return None;
        }

        if col_ix < 4 {
            Some(ColFixed::Left)
        } else {
            None
        }
    }

    fn can_resize_col(&self, col_ix: usize) -> bool {
        return self.col_resize && col_ix > 1;
    }

    fn can_select_col(&self, _: usize) -> bool {
        return self.col_selection;
    }

    fn render_th(&self, col_ix: usize, _cx: &mut ViewContext<Table<Self>>) -> impl IntoElement {
        let th = div().child(self.col_name(col_ix));

        if col_ix >= 3 && col_ix <= 10 {
            th.table_cell_size(self.size)
        } else {
            th
        }
    }

    fn render_tr(
        &self,
        row_ix: usize,
        _: &mut ViewContext<Table<Self>>,
    ) -> gpui::Stateful<gpui::Div> {
        h_flex().id(("table-row", row_ix)).context_menu(|this, _| {
            this.menu("Size Large", Box::new(ChangeSize(Size::Large)))
                .menu("Size Medium", Box::new(ChangeSize(Size::Medium)))
                .menu("Size Small", Box::new(ChangeSize(Size::Small)))
                .menu("Size XSmall", Box::new(ChangeSize(Size::XSmall)))
        })
    }

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        _cx: &mut ViewContext<Table<Self>>,
    ) -> impl IntoElement {
        let stock = self.stocks.get(row_ix).unwrap();
        let col = self.columns.get(col_ix).unwrap();

        match col.id.as_ref() {
            "id" => stock.id.to_string().into_any_element(),
            "name" => stock.name.clone().into_any_element(),
            "symbol" => stock.symbol.clone().into_any_element(),
            "price" => self.render_value_cell(stock.price),
            "change" => self.render_value_cell(stock.change),
            "change_percent" => self.render_value_cell(stock.change_percent),
            "volume" => self.render_value_cell(stock.volume),
            "turnover" => self.render_value_cell(stock.turnover),
            "market_cap" => self.render_value_cell(stock.market_cap),
            "ttm" => self.render_value_cell(stock.ttm),
            "five_mins_ranking" => self.render_value_cell(stock.five_mins_ranking),
            "th60_days_ranking" => stock.th60_days_ranking.to_string().into_any_element(),
            "year_change_percent" => (stock.year_change_percent * 100.0)
                .to_string()
                .into_any_element(),
            "bid" => self.render_value_cell(stock.bid),
            "bid_volume" => self.render_value_cell(stock.bid_volume),
            "ask" => self.render_value_cell(stock.ask),
            "ask_volume" => self.render_value_cell(stock.ask_volume),
            "open" => stock.open.to_string().into_any_element(),
            "prev_close" => stock.prev_close.to_string().into_any_element(),
            "high" => self.render_value_cell(stock.high),
            "low" => self.render_value_cell(stock.low),
            "turnover_rate" => (stock.turnover_rate * 100.0).to_string().into_any_element(),
            "rise_rate" => (stock.rise_rate * 100.0).to_string().into_any_element(),
            "amplitude" => (stock.amplitude * 100.0).to_string().into_any_element(),
            "pe_status" => stock.pe_status.to_string().into_any_element(),
            "pb_status" => stock.pb_status.to_string().into_any_element(),
            "volume_ratio" => self.render_value_cell(stock.volume_ratio),
            "bid_ask_ratio" => self.render_value_cell(stock.bid_ask_ratio),
            "latest_pre_close" => stock.latest_pre_close.to_string().into_any_element(),
            "latest_post_close" => stock.latest_post_close.to_string().into_any_element(),
            "pre_market_cap" => stock.pre_market_cap.to_string().into_any_element(),
            "pre_market_percent" => (stock.pre_market_percent * 100.0)
                .to_string()
                .into_any_element(),
            "pre_market_change" => stock.pre_market_change.to_string().into_any_element(),
            "post_market_cap" => stock.post_market_cap.to_string().into_any_element(),
            "post_market_percent" => (stock.post_market_percent * 100.0)
                .to_string()
                .into_any_element(),
            "post_market_change" => stock.post_market_change.to_string().into_any_element(),
            "float_cap" => stock.float_cap.to_string().into_any_element(),
            "shares" => stock.shares.to_string().into_any_element(),
            "shares_float" => stock.shares_float.to_string().into_any_element(),
            "day_5_ranking" => stock.day_5_ranking.to_string().into_any_element(),
            "day_10_ranking" => stock.day_10_ranking.to_string().into_any_element(),
            "day_30_ranking" => stock.day_30_ranking.to_string().into_any_element(),
            "day_120_ranking" => stock.day_120_ranking.to_string().into_any_element(),
            "day_250_ranking" => stock.day_250_ranking.to_string().into_any_element(),
            _ => "--".to_string().into_any_element(),
        }
    }

    fn can_loop_select(&self) -> bool {
        self.loop_selection
    }

    fn can_move_col(&self, _: usize) -> bool {
        self.col_order
    }

    fn move_col(&mut self, col_ix: usize, to_ix: usize) {
        let col = self.columns.remove(col_ix);
        self.columns.insert(to_ix, col);
    }

    fn col_sort(&self, col_ix: usize) -> Option<ColSort> {
        if !self.col_sort {
            return None;
        }

        self.columns.get(col_ix).and_then(|c| c.sort)
    }

    fn perform_sort(&mut self, col_ix: usize, sort: ColSort, _: &mut ViewContext<Table<Self>>) {
        if !self.col_sort {
            return;
        }

        if let Some(col) = self.columns.get_mut(col_ix) {
            match col.id.as_ref() {
                "id" => self.stocks.sort_by(|a, b| {
                    if sort == ColSort::Ascending {
                        a.id.cmp(&b.id)
                    } else {
                        b.id.cmp(&a.id)
                    }
                }),
                _ => {}
            }
        }
    }

    fn can_load_more(&self) -> bool {
        return !self.loading && !self.is_eof;
    }

    fn load_more_threshold(&self) -> usize {
        150
    }

    fn load_more(&mut self, cx: &mut ViewContext<Table<Self>>) {
        self.loading = true;

        cx.spawn(|view, mut cx| async move {
            // Simulate network request, delay 1s to load data.
            Timer::after(Duration::from_secs(1)).await;

            cx.update(|cx| {
                let _ = view.update(cx, |view, _| {
                    view.delegate_mut().stocks.extend(random_stocks(200));
                    view.delegate_mut().loading = false;
                    view.delegate_mut().is_eof = view.delegate().stocks.len() >= 6000;
                });
            })
        })
        .detach();
    }
}

pub struct TableStory {
    table: View<Table<StockTableDelegate>>,
    num_stocks_input: View<TextInput>,
    stripe: bool,
    refresh_data: bool,
    size: Size,
}

impl super::Story for TableStory {
    fn title() -> &'static str {
        "Table"
    }

    fn description() -> &'static str {
        "A complex data table with selection, sorting, column moving, and loading more."
    }

    fn new_view(cx: &mut WindowContext) -> View<impl gpui::FocusableView> {
        Self::view(cx)
    }

    fn closeable() -> bool {
        false
    }
}

impl gpui::FocusableView for TableStory {
    fn focus_handle(&self, cx: &gpui::AppContext) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl TableStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        // Create the number input field with validation for positive integers
        let num_stocks_input = cx.new_view(|cx| {
            let mut input = TextInput::new(cx)
                .placeholder("Enter number of Stocks to display")
                .validate(|s| s.parse::<usize>().is_ok());
            input.set_text("5000", cx);
            input
        });

        let delegate = StockTableDelegate::new(5000);
        let table = cx.new_view(|cx| Table::new(delegate, cx));

        cx.subscribe(&table, Self::on_table_event).detach();
        cx.subscribe(&num_stocks_input, Self::on_num_stocks_input_change)
            .detach();

        // Spawn a background to random refresh the list
        cx.spawn(move |this, mut cx| async move {
            loop {
                let delay = (80..150).fake::<u64>();
                Timer::after(time::Duration::from_millis(delay)).await;

                this.update(&mut cx, |this, cx| {
                    if !this.refresh_data {
                        return;
                    }

                    this.table.update(cx, |table, _| {
                        table.delegate_mut().stocks.iter_mut().enumerate().for_each(
                            |(i, stock)| {
                                let n = (3..10).fake::<usize>();
                                // update 30% of the stocks
                                if i % n == 0 {
                                    stock.random_update();
                                }
                            },
                        );
                    });
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();

        Self {
            table,
            num_stocks_input,
            stripe: false,
            refresh_data: false,
            size: Size::default(),
        }
    }

    // Event handler for changes in the number input field
    fn on_num_stocks_input_change(
        &mut self,
        _: View<TextInput>,
        event: &InputEvent,
        cx: &mut ViewContext<Self>,
    ) {
        match event {
            // Update when the user presses Enter or the input loses focus
            InputEvent::PressEnter | InputEvent::Blur => {
                let text = self.num_stocks_input.read(cx).text().to_string();
                if let Ok(num) = text.parse::<usize>() {
                    self.table.update(cx, |table, _| {
                        table.delegate_mut().update_stocks(num);
                    });
                    cx.notify();
                }
            }
            _ => {}
        }
    }

    fn toggle_loop_selection(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().loop_selection = *checked;
            cx.notify();
        });
    }

    fn toggle_col_resize(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().col_resize = *checked;
            cx.notify();
        });
    }

    fn toggle_col_order(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().col_order = *checked;
            cx.notify();
        });
    }

    fn toggle_col_sort(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().col_sort = *checked;
            cx.notify();
        });
    }

    fn toggle_col_selection(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().col_selection = *checked;
            cx.notify();
        });
    }

    fn toggle_stripe(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        self.stripe = *checked;
        let stripe = self.stripe;
        self.table.update(cx, |table, cx| {
            table.set_stripe(stripe, cx);
            cx.notify();
        });
    }

    fn toggle_fixed_cols(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        self.table.update(cx, |table, cx| {
            table.delegate_mut().fixed_cols = *checked;
            table.refresh(cx);
            cx.notify();
        });
    }

    fn on_change_size(&mut self, a: &ChangeSize, cx: &mut ViewContext<Self>) {
        self.size = a.0;
        self.table.update(cx, |table, cx| {
            table.set_size(a.0, cx);
            table.delegate_mut().size = a.0;
        });
    }

    fn toggle_refresh_data(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        self.refresh_data = *checked;
        cx.notify();
    }

    fn on_table_event(
        &mut self,
        _: View<Table<StockTableDelegate>>,
        event: &TableEvent,
        _cx: &mut ViewContext<Self>,
    ) {
        match event {
            TableEvent::ColWidthsChanged(col_widths) => {
                println!("Col widths changed: {:?}", col_widths)
            }
            TableEvent::SelectCol(ix) => println!("Select col: {}", ix),
            TableEvent::SelectRow(ix) => println!("Select row: {}", ix),
        }
    }
}

impl Render for TableStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl gpui::IntoElement {
        let delegate = self.table.read(cx).delegate();
        let delegate_size = delegate.rows_count(cx);
        let size = self.size;

        v_flex()
            .on_action(cx.listener(Self::on_change_size))
            .size_full()
            .text_sm()
            .gap_2()
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .flex_wrap()
                    .child(
                        Button::new("size")
                            .compact()
                            .outline()
                            .label(format!("size: {:?}", self.size))
                            .popup_menu(move |menu, _| {
                                menu.menu_with_check(
                                    "Large",
                                    size == Size::Large,
                                    Box::new(ChangeSize(Size::Large)),
                                )
                                .menu_with_check(
                                    "Medium",
                                    size == Size::Medium,
                                    Box::new(ChangeSize(Size::Medium)),
                                )
                                .menu_with_check(
                                    "Small",
                                    size == Size::Small,
                                    Box::new(ChangeSize(Size::Small)),
                                )
                                .menu_with_check(
                                    "XSmall",
                                    size == Size::XSmall,
                                    Box::new(ChangeSize(Size::XSmall)),
                                )
                            }),
                    )
                    .child(
                        Checkbox::new("loop-selection")
                            .label("Loop Selection")
                            .selected(delegate.loop_selection)
                            .on_click(cx.listener(Self::toggle_loop_selection)),
                    )
                    .child(
                        Checkbox::new("col-resize")
                            .label("Column Resize")
                            .selected(delegate.col_resize)
                            .on_click(cx.listener(Self::toggle_col_resize)),
                    )
                    .child(
                        Checkbox::new("col-order")
                            .label("Column Order")
                            .selected(delegate.col_order)
                            .on_click(cx.listener(Self::toggle_col_order)),
                    )
                    .child(
                        Checkbox::new("col-sort")
                            .label("Column Sort")
                            .selected(delegate.col_sort)
                            .on_click(cx.listener(Self::toggle_col_sort)),
                    )
                    .child(
                        Checkbox::new("col-selection")
                            .label("Column Selection")
                            .selected(delegate.col_selection)
                            .on_click(cx.listener(Self::toggle_col_selection)),
                    )
                    .child(
                        Checkbox::new("stripe")
                            .label("Stripe")
                            .selected(self.stripe)
                            .on_click(cx.listener(Self::toggle_stripe)),
                    )
                    .child(
                        Checkbox::new("fixed-cols")
                            .label("Fixed Columns")
                            .selected(delegate.fixed_cols)
                            .on_click(cx.listener(Self::toggle_fixed_cols)),
                    )
                    .child(
                        Checkbox::new("refresh-data")
                            .label("Refresh Data")
                            .selected(self.refresh_data)
                            .on_click(cx.listener(Self::toggle_refresh_data)),
                    ),
            )
            .child(
                h_flex().items_center().gap_2().child(
                    h_flex()
                        .items_center()
                        .gap_1()
                        .child(Label::new("Number of Stocks:"))
                        .child(
                            h_flex()
                                .min_w_32()
                                .child(self.num_stocks_input.clone())
                                .into_any_element(),
                        )
                        .when(delegate.loading, |this| {
                            this.child(h_flex().gap_1().child(Indicator::new()).child("Loading..."))
                        })
                        .child(format!("Total Rows: {}", delegate_size))
                        .when(delegate.is_eof, |this| this.child("All data loaded.")),
                ),
            )
            .child(self.table.clone())
    }
}
