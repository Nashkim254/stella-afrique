use crate::routes::orders::{AdminOrderDetailResponse, OrderLineResponse, OrderSummaryResponse};

pub struct EmailTemplate {
    pub subject: String,
    pub html: String,
    pub text: String,
}

pub fn order_created(order: &OrderSummaryResponse) -> EmailTemplate {
    let preview = format!(
        "Your order {} is confirmed. Total: {} {}.",
        order.order_number, order.currency, order.total_amount
    );

    EmailTemplate {
        subject: format!("Your Stellafrique order {} is confirmed", order.order_number),
        html: render_shell(
            "Order Confirmed",
            &format!("Thank you, {}.", order.customer_name),
            &preview,
            &[
                format!(
                    "<p>We have created your order <strong>{}</strong> and our team will keep you updated as it moves through payment and fulfilment.</p>",
                    escape_html(&order.order_number)
                ),
                render_summary_card(
                    "Order Summary",
                    &order.order_number,
                    &order.currency,
                    &order.total_amount.to_string(),
                    &render_item_rows(&order.items, &order.currency),
                ),
            ],
        ),
        text: render_plain_text(
            &format!("Your Stellafrique order {} is confirmed", order.order_number),
            &[
                format!("Hello {},", order.customer_name),
                format!("Thank you for shopping with Stellafrique. We created order {}.", order.order_number),
                render_plain_items(&order.items, &order.currency),
                format!("Total: {} {}", order.currency, order.total_amount),
                "We will email you again when payment or fulfilment status changes.".to_string(),
            ],
        ),
    }
}

pub fn order_paid(order: &AdminOrderDetailResponse) -> EmailTemplate {
    let preview = format!(
        "Payment received for order {}.",
        order.order_number
    );

    EmailTemplate {
        subject: format!("Payment received for order {}", order.order_number),
        html: render_shell(
            "Payment Received",
            &format!("Hello {},", order.customer_name),
            &preview,
            &[
                format!(
                    "<p>We have received payment for your order <strong>{}</strong>.</p>",
                    escape_html(&order.order_number)
                ),
                render_meta_pair(
                    "Payment method",
                    order.payment_method.as_deref().unwrap_or("Not specified"),
                    "Reference",
                    order.payment_reference.as_deref().unwrap_or("Not specified"),
                ),
                render_summary_card(
                    "Order Summary",
                    &order.order_number,
                    &order.currency,
                    &order.total_amount.to_string(),
                    &render_item_rows(&order.items, &order.currency),
                ),
            ],
        ),
        text: render_plain_text(
            &format!("Payment received for order {}", order.order_number),
            &[
                format!("Hello {},", order.customer_name),
                format!("We have received payment for your order {}.", order.order_number),
                format!(
                    "Payment method: {}",
                    order.payment_method.as_deref().unwrap_or("Not specified")
                ),
                format!(
                    "Reference: {}",
                    order.payment_reference.as_deref().unwrap_or("Not specified")
                ),
                render_plain_items(&order.items, &order.currency),
                format!("Total: {} {}", order.currency, order.total_amount),
            ],
        ),
    }
}

pub fn order_fulfilled(order: &AdminOrderDetailResponse) -> EmailTemplate {
    let preview = format!(
        "Your order {} has been fulfilled.",
        order.order_number
    );

    EmailTemplate {
        subject: format!("Your order {} has been fulfilled", order.order_number),
        html: render_shell(
            "Order Fulfilled",
            &format!("Hello {},", order.customer_name),
            &preview,
            &[
                format!(
                    "<p>Your order <strong>{}</strong> has now been fulfilled and is ready for delivery or collection handling.</p>",
                    escape_html(&order.order_number)
                ),
                render_meta_pair(
                    "Courier",
                    order.shipping_courier.as_deref().unwrap_or("Not specified"),
                    "Tracking",
                    order.tracking_number.as_deref().unwrap_or("Not specified"),
                ),
                render_summary_card(
                    "What was fulfilled",
                    &order.order_number,
                    &order.currency,
                    &order.total_amount.to_string(),
                    &render_item_rows(&order.items, &order.currency),
                ),
            ],
        ),
        text: render_plain_text(
            &format!("Your order {} has been fulfilled", order.order_number),
            &[
                format!("Hello {},", order.customer_name),
                format!(
                    "Your order {} has now been fulfilled and is ready for delivery or collection handling.",
                    order.order_number
                ),
                format!(
                    "Courier: {}",
                    order.shipping_courier.as_deref().unwrap_or("Not specified")
                ),
                format!(
                    "Tracking: {}",
                    order.tracking_number.as_deref().unwrap_or("Not specified")
                ),
                render_plain_items(&order.items, &order.currency),
                format!("Total: {} {}", order.currency, order.total_amount),
            ],
        ),
    }
}

pub fn internal_order_created(order: &OrderSummaryResponse) -> EmailTemplate {
    EmailTemplate {
        subject: format!("New order {} from {}", order.order_number, order.customer_name),
        html: render_internal_shell(
            "New Order",
            &format!(
                "<p><strong>{}</strong> placed order <strong>{}</strong>.</p>",
                escape_html(&order.customer_name),
                escape_html(&order.order_number)
            ),
            &[
                render_meta_pair(
                    "Customer",
                    &format!(
                        "{} ({})",
                        order.customer_name, order.customer_email
                    ),
                    "Status",
                    &format!("{} / {}", order.status, order.payment_status),
                ),
                render_summary_card(
                    "Order Summary",
                    &order.order_number,
                    &order.currency,
                    &order.total_amount.to_string(),
                    &render_item_rows(&order.items, &order.currency),
                ),
            ],
        ),
        text: render_plain_text(
            &format!("New order {} from {}", order.order_number, order.customer_name),
            &[
                format!(
                    "Customer: {} ({})",
                    order.customer_name, order.customer_email
                ),
                format!(
                    "Status: {} / {}",
                    order.status, order.payment_status
                ),
                render_plain_items(&order.items, &order.currency),
                format!("Total: {} {}", order.currency, order.total_amount),
            ],
        ),
    }
}

pub fn internal_order_updated(
    order: &AdminOrderDetailResponse,
    previous_status: &str,
    previous_payment_status: &str,
) -> EmailTemplate {
    EmailTemplate {
        subject: format!(
            "Order {} updated: {} / {}",
            order.order_number, order.status, order.payment_status
        ),
        html: render_internal_shell(
            "Order Updated",
            &format!(
                "<p>Order <strong>{}</strong> changed state.</p>",
                escape_html(&order.order_number)
            ),
            &[
                render_meta_pair(
                    "Fulfilment",
                    &format!("{} → {}", previous_status, order.status),
                    "Payment",
                    &format!("{} → {}", previous_payment_status, order.payment_status),
                ),
                render_meta_pair(
                    "Customer",
                    &format!(
                        "{} ({})",
                        order.customer_name, order.customer_email
                    ),
                    "Total",
                    &format!("{} {}", order.currency, order.total_amount),
                ),
                render_summary_card(
                    "Order Summary",
                    &order.order_number,
                    &order.currency,
                    &order.total_amount.to_string(),
                    &render_item_rows(&order.items, &order.currency),
                ),
            ],
        ),
        text: render_plain_text(
            &format!(
                "Order {} updated: {} / {}",
                order.order_number, order.status, order.payment_status
            ),
            &[
                format!("Fulfilment: {} -> {}", previous_status, order.status),
                format!(
                    "Payment: {} -> {}",
                    previous_payment_status, order.payment_status
                ),
                format!(
                    "Customer: {} ({})",
                    order.customer_name, order.customer_email
                ),
                render_plain_items(&order.items, &order.currency),
                format!("Total: {} {}", order.currency, order.total_amount),
            ],
        ),
    }
}

fn render_shell(title: &str, greeting: &str, preview: &str, sections: &[String]) -> String {
    let body = sections.join("");
    format!(
        concat!(
            "<!doctype html>",
            "<html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">",
            "<title>{}</title></head>",
            "<body style=\"margin:0;background:#f6f3ff;font-family:Arial,Helvetica,sans-serif;color:#151875;\">",
            "<div style=\"max-width:640px;margin:0 auto;padding:32px 18px;\">",
            "<div style=\"background:linear-gradient(135deg,#f3f9ff 0%,#f1f0ff 100%);border-radius:28px;padding:36px;border:1px solid rgba(126,51,224,0.08);box-shadow:0 20px 60px rgba(21,24,117,0.08);\">",
            "<div style=\"display:inline-block;padding:8px 14px;border-radius:999px;background:#ffffff;color:#fb2e86;font-size:12px;font-weight:700;letter-spacing:0.08em;text-transform:uppercase;\">Stellafrique</div>",
            "<h1 style=\"margin:18px 0 12px;font-size:32px;line-height:1.1;color:#151875;\">{}</h1>",
            "<p style=\"margin:0 0 8px;font-size:18px;line-height:1.6;color:#151875;\">{}</p>",
            "<p style=\"margin:0 0 28px;font-size:14px;line-height:1.7;color:#8a8fb9;\">{}</p>",
            "{}",
            "<div style=\"margin-top:28px;padding-top:22px;border-top:1px solid rgba(138,143,185,0.24);font-size:13px;line-height:1.7;color:#8a8fb9;\">",
            "<p style=\"margin:0;\">Need help with your order? Reply to this email and the Stellafrique team will get back to you.</p>",
            "</div></div></div></body></html>"
        ),
        escape_html(title),
        escape_html(title),
        escape_html(greeting),
        escape_html(preview),
        body
    )
}

fn render_internal_shell(title: &str, intro_html: &str, sections: &[String]) -> String {
    let body = sections.join("");
    format!(
        concat!(
            "<!doctype html>",
            "<html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">",
            "<title>{}</title></head>",
            "<body style=\"margin:0;background:#f7f8fc;font-family:Arial,Helvetica,sans-serif;color:#151875;\">",
            "<div style=\"max-width:680px;margin:0 auto;padding:28px 18px;\">",
            "<div style=\"background:#ffffff;border-radius:24px;padding:32px;border:1px solid rgba(21,24,117,0.08);box-shadow:0 18px 48px rgba(21,24,117,0.08);\">",
            "<p style=\"margin:0 0 10px;font-size:12px;letter-spacing:0.08em;text-transform:uppercase;color:#7e33e0;font-weight:700;\">Stellafrique Internal</p>",
            "<h1 style=\"margin:0 0 12px;font-size:30px;line-height:1.1;color:#151875;\">{}</h1>",
            "{}",
            "{}",
            "</div></div></body></html>"
        ),
        escape_html(title),
        escape_html(title),
        intro_html,
        body
    )
}

fn render_summary_card(
    heading: &str,
    order_number: &str,
    currency: &str,
    total_amount: &str,
    rows: &str,
) -> String {
    format!(
        concat!(
            "<div style=\"background:#ffffff;border-radius:22px;padding:24px;border:1px solid rgba(21,24,117,0.08);\">",
            "<div style=\"display:flex;justify-content:space-between;gap:16px;flex-wrap:wrap;margin-bottom:18px;\">",
            "<div><p style=\"margin:0 0 6px;font-size:12px;letter-spacing:0.08em;text-transform:uppercase;color:#7e33e0;font-weight:700;\">{}</p>",
            "<p style=\"margin:0;font-size:14px;color:#151875;\">Order <strong>{}</strong></p></div>",
            "<div style=\"text-align:right;\"><p style=\"margin:0 0 6px;font-size:12px;letter-spacing:0.08em;text-transform:uppercase;color:#7e33e0;font-weight:700;\">Total</p>",
            "<p style=\"margin:0;font-size:20px;font-weight:700;color:#fb2448;\">{} {}</p></div></div>",
            "<div>{}</div></div>"
        ),
        escape_html(heading),
        escape_html(order_number),
        escape_html(currency),
        escape_html(total_amount),
        rows
    )
}

fn render_meta_pair(left_label: &str, left_value: &str, right_label: &str, right_value: &str) -> String {
    format!(
        concat!(
            "<div style=\"display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:14px;margin:0 0 20px;\">",
            "<div style=\"background:#ffffff;border-radius:18px;padding:18px;border:1px solid rgba(21,24,117,0.08);\">",
            "<p style=\"margin:0 0 6px;font-size:12px;letter-spacing:0.08em;text-transform:uppercase;color:#7e33e0;font-weight:700;\">{}</p>",
            "<p style=\"margin:0;color:#151875;\">{}</p></div>",
            "<div style=\"background:#ffffff;border-radius:18px;padding:18px;border:1px solid rgba(21,24,117,0.08);\">",
            "<p style=\"margin:0 0 6px;font-size:12px;letter-spacing:0.08em;text-transform:uppercase;color:#7e33e0;font-weight:700;\">{}</p>",
            "<p style=\"margin:0;color:#151875;\">{}</p></div></div>"
        ),
        escape_html(left_label),
        escape_html(left_value),
        escape_html(right_label),
        escape_html(right_value)
    )
}

fn render_item_rows(items: &[OrderLineResponse], currency: &str) -> String {
    items
        .iter()
        .map(|item| {
            let details = [item.variant_name.as_deref(), item.size.as_deref(), item.color.as_deref()]
                .into_iter()
                .flatten()
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" • ");

            format!(
                concat!(
                    "<div style=\"padding:14px 0;border-top:1px solid rgba(138,143,185,0.18);\">",
                    "<div style=\"display:flex;justify-content:space-between;gap:16px;align-items:flex-start;\">",
                    "<div><p style=\"margin:0 0 4px;font-size:15px;font-weight:700;color:#151875;\">{}</p>",
                    "<p style=\"margin:0 0 4px;font-size:13px;color:#8a8fb9;\">{} </p>",
                    "<p style=\"margin:0;font-size:13px;color:#8a8fb9;\">Qty {}</p></div>",
                    "<p style=\"margin:0;font-size:15px;font-weight:700;color:#151875;\">{} {}</p>",
                    "</div></div>"
                ),
                escape_html(&item.product_name),
                escape_html(&details),
                item.quantity,
                escape_html(currency),
                item.line_total
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_plain_items(items: &[OrderLineResponse], currency: &str) -> String {
    let rows = items
        .iter()
        .map(|item| {
            let details = [item.variant_name.as_deref(), item.size.as_deref(), item.color.as_deref()]
                .into_iter()
                .flatten()
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" / ");

            if details.is_empty() {
                format!(
                    "- {} x{} ({} {})",
                    item.product_name, item.quantity, currency, item.line_total
                )
            } else {
                format!(
                    "- {} [{}] x{} ({} {})",
                    item.product_name, details, item.quantity, currency, item.line_total
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("Items:\n{}", rows)
}

fn render_plain_text(subject: &str, paragraphs: &[String]) -> String {
    let mut lines = vec![subject.to_string(), String::new()];
    lines.extend(paragraphs.iter().cloned());
    lines.join("\n\n")
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
