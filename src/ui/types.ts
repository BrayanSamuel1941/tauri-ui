export type Align = 'start' | 'center' | 'end' | 'stretch';

export interface BaseNode {
    type: string;
    id?: string;
    align?: Align;
    visible_when_flag?: string;
    visible_when?: boolean;
}

export interface TextNode extends BaseNode {
    type: 'text';
    text: string;
    size?: number;
    bold?: boolean;
    color?: string;
    text_from_input_id?: string;
    text_template?: string;
}

export interface ButtonNode extends BaseNode {
    type: 'button';
    text: string;
    on_click?: string;
    tint?: string;
    text_color?: string;
    icon?: string;
    enabled?: boolean;
    enable_when_input_id?: string;
    enable_when_min_cents?: number;
}

export interface SpacerNode extends BaseNode {
    type: 'spacer';
    height?: number;
}

export interface ScrollTextNode extends BaseNode {
    type: 'scroll';
    text: string;
    color?: string;
    padding?: number;
    weight?: number;
}

export interface ColumnNode extends BaseNode {
    type: 'column';
    background?: string;
    padding?: number;
    gap?: number;
    children: UiNode[];
}

// NUEVO: logo
export interface LogoNode extends BaseNode {
    type: 'logo';
    width?: number;
    height?: number;
}

// NUEVO: input_money
export interface InputMoneyNode extends BaseNode {
    type: 'input_money';
    hint?: string;
    currency?: string;
    value?: string;
}

// ⬇️ agrega debajo de InputMoneyNode
export interface InputTextNode extends BaseNode {
    type: 'input_text';
    hint?: string;
    value?: string;
}

export interface InputPasswordNode extends BaseNode {
    type: 'input_password';
    hint?: string;
    value?: string;
}

// ⬇️ añade ambos al union UiNode
export type UiNode =
    | TextNode
    | ButtonNode
    | SpacerNode
    | ScrollTextNode
    | ColumnNode
    | LogoNode
    | InputMoneyNode
    | InputTextNode       // NUEVO
    | InputPasswordNode;  // NUEVO

export interface UiLayout {
    background?: string;
    root: UiNode;
    customer_display?: {
        text?: string;
        size?: number;
        align?: Align;
        use_logo?: boolean;
        bg_color?: string;
    };
}
